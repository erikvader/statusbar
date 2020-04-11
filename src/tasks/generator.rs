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

macro_rules! to_rer {
    ($e:expr) => {
        $e.map_err(|_| ExitReason::Error)
    }
}

pub mod echogen;
pub mod ramgen;
pub mod cpugen;
pub mod timegen;
pub mod netgen;
pub mod diskgen;
pub mod tempgen;

use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::time::delay_for;
use tokio::select;
use core::time::Duration;
use async_trait::async_trait;
pub use super::{ExitReason,Msg};

pub type Result<X> = std::result::Result<X, ExitReason>;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub enum GenType {
    CPU = 0,
    RAM,
    ECHO,
    TIME,
    NET,
    DISK,
    TEMP,
}

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
pub struct GenId {
    gen: GenType,
    id: u8
}

pub struct GenArg {
    pub timeout: Option<u64>,
    pub arg: Option<String>
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

#[async_trait]
pub trait Generator {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>,
                   from_pipo: mpsc::Receiver<String>,
                   id: GenId,
                   arg: Option<GenArg>,
                   name: String) -> ExitReason;
}

#[async_trait]
pub trait TimerGenerator {
    async fn init(&mut self, _arg: &Option<GenArg>) -> Result<()> {Ok(())}
    async fn update(&mut self) -> Result<()>;
    fn display(&self, name: &str) -> Result<String>;
    async fn finalize(&mut self) -> Result<()> {Ok(())}
    async fn on_msg(&mut self, _msg: String) -> Result<bool> {Ok(false)}
    fn get_delay(&self, arg: &Option<GenArg>) -> u64 {
        arg.as_ref().and_then(|ga| ga.timeout).unwrap_or(5)
    }
}

#[async_trait]
impl<G: TimerGenerator + Sync + Send> Generator for G {
    async fn start(&mut self,
                   to_printer: broadcast::Sender<Msg>,
                   mut from_pipo: mpsc::Receiver<String>,
                   id: GenId,
                   arg: Option<GenArg>,
                   name: String) -> ExitReason
    {
        unwrap_er!(self.init(&arg).await);
        let mut run_update = true;
        let mut delayer = delay_for(Duration::from_secs(0));
        loop {
            if run_update {
                unwrap_er!(self.update().await);
                run_update = false;
                let delay = Duration::from_secs(self.get_delay(&arg));
                delayer.reset(tokio::time::Instant::now() + delay);
            }
            let s = unwrap_er!(self.display(&name));
            if let Err(_) = to_printer.send((id, s)) {
                break;
            }
            let msg = select! {
                _ = &mut delayer => {
                    run_update = true;
                    None
                },
                msg = from_pipo.recv() => match msg {
                    None => break,
                    Some(m) => Some(m)
                }
            };
            if let Some(m) = msg {
                run_update = unwrap_er!(self.on_msg(m).await);
            }
        }
        unwrap_er!(self.finalize().await);
        ExitReason::Normal
    }
}

pub fn genid_to_generator(id: GenId) -> Box<dyn Generator + Send> {
    match id.gen {
        GenType::ECHO => Box::new(echogen::EchoGen),
        GenType::RAM  => Box::new(ramgen::RamGen::new()),
        GenType::CPU  => Box::new(cpugen::CpuGen::new()),
        GenType::TIME => Box::new(timegen::TimeGen::new()),
        GenType::NET  => Box::new(netgen::NetGen::new()),
        GenType::DISK => Box::new(diskgen::DiskGen::new()),
        GenType::TEMP => Box::new(tempgen::TempGen::new()),
    }
}
