pub mod echogen;
pub mod ramgen;
pub mod cpugen;

use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::time::delay_for;
use tokio::select;
use core::time::Duration;
use async_trait::async_trait;
use crate::constants::*;
use super::ExitReason;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub enum GenType {
    CPU = 0,
    RAM,
    ECHO
}

#[derive(Clone,Copy,PartialEq,Eq,Hash)]
pub struct GenId {
    gen: GenType,
    id: u8
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
                   arg: Option<String>) -> ExitReason;
}

// TODO: incorporate ExitReason somehow
#[async_trait]
pub trait TimerGenerator {
    async fn init(&mut self, _arg: Option<String>) {}
    async fn update(&mut self) -> String;
    async fn finalize(&mut self) {}
    async fn on_msg(&mut self, _msg: String) {}
    fn get_delay(&self) -> u64 { 5 }
}

#[async_trait]
impl<G: TimerGenerator + Sync + Send> Generator for G {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>, mut from_pipo: mpsc::Receiver<String>, id: GenId, arg: Option<String>) -> ExitReason {
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
        ExitReason::Normal
    }
}

pub fn genid_to_generator(id: GenId) -> Box<dyn Generator + Send> {
    match id.gen {
        GenType::ECHO => Box::new(echogen::EchoGen),
        GenType::RAM => Box::new(ramgen::RamGen::new()),
        GenType::CPU => Box::new(cpugen::CpuGen::new())
    }
}
