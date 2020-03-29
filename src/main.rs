use std::future::Future;
use nix::unistd::{mkfifo, unlink};
use nix::sys::stat;
use nix::Error::Sys;
use nix::errno::Errno::EEXIST;
use std::collections::HashSet;
use std::collections::HashMap;
use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast::{self, RecvError};
use tokio::task::JoinHandle;
use tokio::time::delay_for;
use tokio::select;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::signal::unix::{signal, SignalKind};
use tokio::process::Command;
use sysinfo::{SystemExt,ProcessorExt};
use itertools::Itertools;
use core::time::Duration;
use async_trait::async_trait;

// TODO: göra en trait kanske för att skapa stränger att skicka till dzen?
// TODO: tänk på hur programmet ska startas. Alltid daemoniza? Hantera SIGHUP?
// TODO: hantera att en arbiträr task kan krascha. Få alla tasks att returna ExitReason
// https://docs.rs/futures/0.3.4/futures/stream/struct.FuturesUnordered.html

enum ExitReason {
    Signal,
    Error,
    SignalError,
    Normal
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
enum GenType {
    CPU = 0,
    RAM,
    ECHO
}

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
struct GenId {
    gen: GenType,
    id: u8
}

impl GenId {
    fn new(gen: GenType, id: u8) -> Self {
        GenId{gen: gen, id: id}
    }
    fn from_gen(gen: GenType) -> Self {
        Self::new(gen, gen as u8)
    }
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

#[async_trait]
trait Generator {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>, from_pipo: mpsc::Receiver<String>, id: GenId, arg: Option<String>);
}

#[async_trait]
trait TimerGenerator {
    async fn init(&mut self, _arg: Option<String>) {}
    async fn update(&mut self) -> String;
    async fn finalize(&mut self) {}
    async fn on_msg(&mut self, _msg: String) {}
    fn get_delay(&self) -> u64 { 5 }
}

#[async_trait]
impl<G: TimerGenerator + Sync + Send> Generator for G {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>, mut from_pipo: mpsc::Receiver<String>, id: GenId, arg: Option<String>) {
        self.init(arg).await;
        let delay = Duration::from_secs(self.get_delay());
        loop {
            let s = self.update().await;
            if let Err(_) = to_printer.send((id, s)) {
                break;
            }
            let msg = select! {
                _ = delay_for(delay) => None,
                msg = from_pipo.recv() => match msg {
                    None => break,
                    Some(m) => Some(m)
                }
            };
            if let Some(m) = msg {
                self.on_msg(m).await;
            }
        }
        self.finalize().await;
    }
}

struct EchoGen;
struct CpuGen{sys: sysinfo::System, detailed: bool}
struct RamGen{sys: sysinfo::System}

const FIFO_PATH: &str = "/tmp/statusbar_fifo";
const MPSC_SIZE: usize = 32;

type Msg = (GenId, String);

struct BarConfig {
    left: Vec<GenId>,
    right: Vec<GenId>,
    xinerama: u32,
    tray: bool,
    separator: String,
}

struct SetupConfig {
    arguments: HashMap<GenId, String>,
    names: HashMap<GenId, String>,
    bars: Vec<BarConfig>,
    id: u8
}

impl SetupConfig {
    fn new() -> Self {
        SetupConfig{
            arguments: HashMap::new(),
            names: HashMap::new(),
            bars: Vec::new(),
            id: 100 // TODO: count the elements in GenType
        }
    }

    fn create_module(&mut self, gen: GenType, arg: Option<String>) -> GenId {
        if let Some(a) = arg {
            let id = GenId::new(gen, self.id);
            self.id += 1;
            self.arguments.insert(id, a);
            id
        } else {
            GenId::from_gen(gen)
        }
    }

    fn name_module(&mut self, id: GenId, name: String) {
        self.names.insert(id, name);
    }

    fn add_bar(&mut self, bar: BarConfig) {
        self.bars.push(bar);
    }

    fn get_arg(&self, id: GenId) -> Option<&String> {
        self.arguments.get(&id)
    }

    fn get_name(&self, id: GenId) -> Option<&String> {
        self.names.get(&id)
    }

    fn iter(&self) -> impl Iterator<Item=&GenId> {
        self.bars.iter().flat_map(|b| b.iter()).unique()
    }

    fn take_bars(self) -> Vec<BarConfig> {
        self.bars
    }
}

impl BarConfig {
    fn new(xinerama: u32) -> Self {
        Self{
            left: Vec::new(),
            right: Vec::new(),
            xinerama: xinerama,
            separator: " | ".to_string(),
            tray: false
        }
    }

    fn add_left(&mut self, id: GenId) {
        self.left.push(id);
    }

    fn add_right(&mut self, id: GenId) {
        self.right.push(id);
    }

    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }

    fn iter(&self) -> impl Iterator<Item=&GenId> {
        self.left.iter().chain(self.right.iter())
    }
}

async fn start(setup: SetupConfig) -> ExitReason {
    let mut tasks = Vec::new();

    let pipo_handle = {
        let mut pipo_map = HashMap::new();

        let (broad_send, _) = broadcast::channel(MPSC_SIZE);

        for g in setup.iter() {
            let (pipo_send, pipo_recv) = mpsc::channel(MPSC_SIZE);
            let bs = broad_send.clone();
            let a = setup.get_arg(*g).cloned();
            let gg = *g;
            tasks.push(tokio::spawn(async move {
                let mut gen = genid_to_generator(gg);
                gen.start(bs, pipo_recv, gg, a).await;
            }));
            if let Some(_) = pipo_map.insert(setup.get_name(*g).cloned().unwrap_or(g.to_string()), pipo_send) {
                panic!("some generators have the same name!");
            }
        }

        for b in setup.take_bars().into_iter() {
            tasks.push(tokio::spawn(printer(broad_send.subscribe(), b)));
        }

        tokio::spawn(pipo_reader(pipo_map))
    };

    for t in tasks {
        match t.await {
            _ => ()
        }
    }

    // TODO: remove
    println!("all tasks have finished, exiting...");

    pipo_handle.await.unwrap()
}

async fn pipo_reader(mut gens: HashMap<String, mpsc::Sender<String>>) -> ExitReason {
    // create pipe
    match mkfifo(FIFO_PATH, stat::Mode::S_IRWXU) {
        Ok(()) => (),
        Err(Sys(errno)) if errno == EEXIST => (),
        Err(e) => {
            eprintln!("couldn't create pipo at {} because '{}'", FIFO_PATH, e);
            return ExitReason::Error;
        }
    };

    let mut int_stream = signal(SignalKind::interrupt()).unwrap();
    let mut term_stream = signal(SignalKind::terminate()).unwrap();

    // read pipe
    let main_loop = async {
        'outer: loop {
            // TODO: handle if this fails
            let f = File::open(FIFO_PATH).await.unwrap();
            let mut reader = BufReader::new(f);

            let mut buf = String::new();
            while let Ok(c) = reader.read_line(&mut buf).await {
                if c == 0 {
                    break;
                }
                let content = buf.trim_end();

                let (gid, msg) =
                    match content.match_indices(" ").next() {
                        Some((i, _)) => (&content[..i], &content[i+1..]),
                        _ => (content, "")
                    };

                if gid == "EXIT" {
                    break 'outer
                }

                if let Some(send) = gens.get_mut(gid) {
                    match send.try_send(msg.to_string()) {
                        Err(mpsc::error::TrySendError::Closed(_)) => break 'outer,
                        _ => ()
                    }
                }
                buf.clear();
            }
        }
    };

    let was_signal = select! {
        _ = int_stream.recv() => true,
        _ = term_stream.recv() => true,
        _ = main_loop => false
    };

    // remove pipe
    if let Err(e) = unlink(FIFO_PATH) {
        eprintln!("Couldn't remove pipe at {} because '{}'", FIFO_PATH, e);
    }

    if was_signal {
        ExitReason::Signal
    } else {
        ExitReason::Normal
    }
}

fn genid_to_generator(id: GenId) -> Box<dyn Generator + Send> {
    match id.gen {
        GenType::ECHO => Box::new(EchoGen),
        GenType::RAM => Box::new(RamGen::new()),
        GenType::CPU => Box::new(CpuGen::new())
    }
}

#[async_trait]
impl Generator for EchoGen {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>, mut from_pipo: mpsc::Receiver<String>, id: GenId, _arg: Option<String>) {
        while let Some(inp) = from_pipo.recv().await {
            if let Err(_) = to_printer.send((id, inp)) {
                break;
            }
        }
    }
}

impl CpuGen {
    fn new() -> Self {
        CpuGen{sys: sysinfo::System::new(), detailed: false}
    }
}

impl RamGen {
    fn new() -> Self {
        RamGen{sys: sysinfo::System::new()}
    }
}

#[async_trait]
impl TimerGenerator for CpuGen {
    fn get_delay(&self) -> u64 { 3 }

    async fn init(&mut self, arg: Option<String>) {
        if let Some(a) = arg {
            if a == "detailed" {
                self.detailed = true;
            }
        }
    }

    async fn update(&mut self) -> String {
        self.sys.refresh_cpu();
        if self.detailed {
            self.sys.get_processors()
                .iter()
                .map(|p| format!("{:0>2}", p.get_cpu_usage().round()))
                .join("/")
        } else {
            self.sys.get_global_processor_info().get_cpu_usage().round().to_string()
        }
    }

    async fn on_msg(&mut self, _msg: String) {
        self.detailed = !self.detailed;
    }
}

#[async_trait]
impl TimerGenerator for RamGen {
    fn get_delay(&self) -> u64 { 7 }
    async fn update(&mut self) -> String {
        self.sys.refresh_memory();
        let usage = ((self.sys.get_used_memory() as f64 / self.sys.get_total_memory() as f64) * 100.0).round();
        format!("{}%", usage)
    }
}

async fn printer(mut recv: broadcast::Receiver<Msg>, config: BarConfig) {
    let mut output = HashMap::<GenId, String>::new();
    for id in config.iter() {
        output.insert(*id, "".to_string());
    }

    // TODO: check if dzen gets killed early
    let dzen = Command::new("dzen2")
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .args(&["-fg", "#dfdfdf"])
        .args(&["-bg", "#333333"])
        .args(&["-fn", "xft:Ubuntu Mono:pixelsize=14:antialias=true:hinting=true"])
        // .args(&["-ta", "l"])
        .args(&["-h", "26"])
        .args(&["-xs", &config.xinerama.to_string()])
        .spawn()
        // TODO: gracefully terminate whole program if this fails.
        // maybe return an Err and have start react to it?
        .expect("dzen couldn't be spawned");

    let mut stdin = dzen.stdin.unwrap();

    loop {
        match recv.recv().await {
            Err(RecvError::Lagged(_)) => continue,
            Err(_) => break,
            Ok((id, msg)) => {
                if !output.contains_key(&id) {
                    continue;
                }

                *output.get_mut(&id).unwrap() = msg;

                let mut first = true;
                let mut buf = String::new();
                for c in config.iter() {
                    let x = output.get(c).unwrap();
                    if x.is_empty() {
                        continue;
                    }
                    if !first {
                        buf.push_str(config.separator.as_str());
                    }
                    buf.push_str(x);
                    first = false;
                }
                buf.push_str("\n");
                // TODO: handle this error
                stdin.write_all(buf.as_bytes()).await.expect("couldn't write");
            }
        }
    }
}

fn main() {
    let mut setup = SetupConfig::new();
    let mut bar1 = BarConfig::new(1);
    let c = setup.create_module(GenType::CPU, Some("detailed".to_string()));
    setup.name_module(c, "cpu".to_string());
    bar1.add_left(c);
    bar1.add_right(setup.create_module(GenType::RAM, None));
    setup.add_bar(bar1);

    let mut bar2 = BarConfig::new(2);
    let echo = setup.create_module(GenType::ECHO, None);
    setup.name_module(echo, "adina".to_string());
    bar2.add_left(echo);
    setup.add_bar(bar2);

    // NOTE: explicitly creating and shutting down a runtime like this
    // is required because of https://github.com/tokio-rs/tokio/issues/2318
    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    let reason = runtime.block_on(start(setup));

    // NOTE: a non-zero timeout shouldn't be needed because we should
    // exit _only_ if all tasks have already exited, but just to be
    // safe
    runtime.shutdown_timeout(Duration::from_secs(1));

    std::process::exit(match reason {
        ExitReason::Normal => 0,
        ExitReason::Error  => 1,
        ExitReason::Signal => 2,
        ExitReason::SignalError => 3
    });
}
