pub mod echogen;
pub mod ramgen;
pub mod cpugen;
pub mod timegen;

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
    async fn update(&mut self, name: &str) -> Result<String>;
    async fn finalize(&mut self) -> Result<()> {Ok(())}
    async fn on_msg(&mut self, _msg: String) -> Result<()> {Ok(())}
    fn get_delay(&self, arg: &Option<GenArg>) -> u64 {
        arg.as_ref().and_then(|ga| ga.timeout).unwrap_or(5)
    }
}

macro_rules! ERTry {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(er) => {
                return er;
            }
        }
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
        ERTry!(self.init(&arg).await);
        loop {
            let s = ERTry!(self.update(&name).await);
            if let Err(_) = to_printer.send((id, s)) {
                break;
            }
            let delay = Duration::from_secs(self.get_delay(&arg));
            let msg = select! {
                _ = delay_for(delay) => None,
                msg = from_pipo.recv() => match msg {
                    None => break,
                    Some(m) => Some(m)
                }
            };
            if let Some(m) = msg {
                ERTry!(self.on_msg(m).await);
            }
        }
        ERTry!(self.finalize().await);
        ExitReason::Normal
    }
}

pub fn genid_to_generator(id: GenId) -> Box<dyn Generator + Send> {
    match id.gen {
        GenType::ECHO => Box::new(echogen::EchoGen),
        GenType::RAM => Box::new(ramgen::RamGen::new()),
        GenType::CPU => Box::new(cpugen::CpuGen::new()),
        GenType::TIME => Box::new(timegen::TimeGen::new()),
    }
}
