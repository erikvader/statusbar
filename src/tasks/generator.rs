macro_rules! unwrap_er {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(er) => {
                return er;
            }
        }
    }
}

pub mod echogen;
pub mod ramgen;
pub mod cpugen;
pub mod timegen;
pub mod netgen;
pub mod diskgen;
pub mod tempgen;
pub mod ipgen;
pub mod onegen;
pub mod batgen;

use dbus_tokio::connection::IOResource;
use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::time::delay_for;
use futures::stream::StreamExt;
use tokio::select;
use core::time::Duration;
use async_trait::async_trait;
pub use super::{ExitReason,Msg};
use dbus::nonblock as DN;
use std::sync::Arc;
use futures::stream::{select_all};
use either::{Left,Right};
use crate::dzen_format::DzenBuilder;

pub type Result<X> = std::result::Result<X, ExitReason>;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
#[allow(dead_code)]
pub enum GenType {
    CPU = 0,
    RAM,
    ECHO,
    TIME,
    NET,
    DISK,
    TEMP,
    IP,
    ONE,
    BAT,
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct GenId {
    pub gen: GenType,
    pub id: u8
}

#[derive(PartialEq)]
pub struct GenArg {
    pub timeout: Option<u64>,
    pub arg: Option<String>,
    pub prepend: Option<DzenBuilder<'static>>,
}

impl GenId {
    pub fn new(gen: GenType, id: u8) -> Self {
        GenId{gen: gen, id: id}
    }
    pub fn from_gen(gen: GenType) -> Self {
        Self::new(gen, gen as u8)
    }
    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl GenArg {
    pub fn get_builder(&self) -> DzenBuilder<'_> {
        self.prepend.as_ref().map_or_else(|| DzenBuilder::new(), |b| b.clone())
    }

    pub fn empty() -> Self {
        GenArg {
            timeout: None,
            arg: None,
            prepend: None,
        }
    }
}

struct TimerWrap<T>(T);
struct DBusWrap<T>(T);

#[async_trait]
pub trait Generator {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>,
                   from_pipo: mpsc::Receiver<String>,
                   id: GenId,
                   arg: GenArg,
                   name: String) -> ExitReason;
}

#[async_trait]
pub trait TimerGenerator {
    async fn init(&mut self, _arg: &GenArg) -> Result<()> {Ok(())}
    async fn update(&mut self) -> Result<()>;
    fn display(&self, name: &str, arg: &GenArg) -> Result<String>;
    async fn finalize(&mut self) -> Result<()> {Ok(())}
    async fn on_msg(&mut self, _msg: String) -> Result<bool> {Ok(false)}
    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(5)
    }
}

#[async_trait]
impl<G: TimerGenerator + Sync + Send> Generator for TimerWrap<G> {
    async fn start(&mut self,
                   to_printer: broadcast::Sender<Msg>,
                   mut from_pipo: mpsc::Receiver<String>,
                   id: GenId,
                   arg: GenArg,
                   name: String) -> ExitReason
    {
        unwrap_er!(self.0.init(&arg).await);
        let mut run_update = true;
        let mut delayer = delay_for(Duration::from_secs(0));
        let reason = loop {
            if run_update {
                unwrap_er!(self.0.update().await);
                run_update = false;
                let delay = Duration::from_secs(self.0.get_delay(&arg));
                delayer.reset(tokio::time::Instant::now() + delay);
            }
            let s = unwrap_er!(self.0.display(&name, &arg));
            if let Err(_) = to_printer.send((id, s)) {
                break ExitReason::Error;
            }
            let msg = select! {
                _ = &mut delayer => {
                    run_update = true;
                    None
                },
                msg = from_pipo.recv() => match msg {
                    None => break ExitReason::Normal,
                    Some(m) => Some(m)
                }
            };
            if let Some(m) = msg {
                run_update = unwrap_er!(self.0.on_msg(m).await);
            }
        };
        unwrap_er!(self.0.finalize().await);
        reason
    }
}

#[async_trait]
pub trait DBusGenerator {
    fn get_connection(&self) -> Result<(IOResource<DN::SyncConnection>, Arc<DN::SyncConnection>)>;
    async fn init(&mut self, _arg: &GenArg, _conn: Arc<DN::SyncConnection>) -> Result<()> {Ok(())}
    async fn update(&mut self, conn: Arc<DN::SyncConnection>, name: &str, arg: &GenArg) -> Result<String>;
    fn interesting_signals(&self) -> Vec<dbus::message::MatchRule<'static>> {vec!()}
    async fn handle_signal(&mut self, _sig: usize, _data: dbus::message::Message) -> Result<()> {Ok(())}
    async fn handle_msg(&mut self, _msg: String) -> Result<()> {Ok(())}
}

#[async_trait]
impl<G> Generator for DBusWrap<G>
where G: DBusGenerator + Sync + Send
{
    async fn start(
        &mut self,
        to_printer: broadcast::Sender<Msg>,
        mut from_pipo: mpsc::Receiver<String>,
        id: GenId,
        arg: GenArg,
        name: String
    ) -> ExitReason
    {
        let (mut resource, conn) = unwrap_er!(self.0.get_connection());

        // declare main loop
        let main_loop = async {
            self.0.init(&arg, conn.clone()).await?;

            let int_sig = self.0.interesting_signals();
            let mut sigs_streams = Vec::new();
            let mut sigs_tokens = Vec::new();
            for (i, s) in int_sig.into_iter().enumerate() {
                let (mm, stream) = conn.add_match(s).await?.msg_stream();
                sigs_streams.push(stream.map(move |x| (i, x)));
                sigs_tokens.push(mm);
            }

            let mut stream = select_all(sigs_streams);

            let res = loop {
                let s = self.0.update(conn.clone(), name.as_str(), &arg).await?;
                if let Err(_) = to_printer.send((id, s)) {
                    break Err(ExitReason::Error);
                }

                let msg = select! {
                    msg = from_pipo.recv() => match msg {
                        None => break Ok(()),
                        Some(s) => Right(s),
                    },
                    x = stream.next(), if !sigs_tokens.is_empty() => Left(x)
                };

                match msg {
                    Left(Some((sig_num, data))) => {
                        self.0.handle_signal(sig_num, data).await?;
                    }
                    Right(s) => {
                        self.0.handle_msg(s).await?;
                    }
                    Left(None) => {
                        eprintln!("can't wait on more DBus signals?");
                        break Err(ExitReason::Error);
                    }
                }
            };

            for t in sigs_tokens.into_iter() {
                conn.remove_match(t.token()).await?;
            }

            res
        };

        // wait for main loop or dbus disconnect
        let ret = tokio::select! {
            err = &mut resource => {
                eprintln!("dbus connection lost. '{}'", err);
                Err(ExitReason::Error)
            },
            ret = main_loop => ret
        };

        match ret {
            Ok(_)   => ExitReason::Normal,
            Err(er) => er
        }
    }
}

pub fn genid_to_generator(id: GenId) -> Box<dyn Generator + Send> {
    match id.gen {
        GenType::ECHO => Box::new(echogen::EchoGen),
        GenType::RAM  => Box::new(TimerWrap(ramgen::RamGen::new())),
        GenType::CPU  => Box::new(TimerWrap(cpugen::CpuGen::new())),
        GenType::TIME => Box::new(TimerWrap(timegen::TimeGen::new())),
        GenType::NET  => Box::new(TimerWrap(netgen::NetGen::new())),
        GenType::DISK => Box::new(TimerWrap(diskgen::DiskGen::new())),
        GenType::TEMP => Box::new(TimerWrap(tempgen::TempGen::new())),
        GenType::IP   => Box::new(DBusWrap(ipgen::IpGen::new())),
        GenType::ONE  => Box::new(onegen::OneGen::new()),
        GenType::BAT  => Box::new(TimerWrap(batgen::BatGen::new())),
    }
}
