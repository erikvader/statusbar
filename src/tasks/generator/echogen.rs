use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use super::*;
use crate::tasks::ExitReason;
use crate::constants::*;

pub struct EchoGen;

#[async_trait]
impl Generator for EchoGen {
    async fn start(&mut self, to_printer: broadcast::Sender<Msg>, mut from_pipo: mpsc::Receiver<String>, id: GenId, _arg: Option<String>) -> ExitReason {
        while let Some(inp) = from_pipo.recv().await {
            if let Err(_) = to_printer.send((id, inp)) {
                return ExitReason::Error;
            }
        }
        ExitReason::Normal
    }
}
