use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use super::*;
use crate::tasks::ExitReason;
use crate::dzen_format::external::fix_dzen_string;

pub struct EchoGen;

#[async_trait]
impl Generator for EchoGen {
    async fn start(&mut self,
                   to_printer: broadcast::Sender<Msg>,
                   mut from_pipo: mpsc::Receiver<String>,
                   id: GenId,
                   arg: GenArg,
                   _name: String) -> ExitReason
    {
        while let Some(inp) = from_pipo.recv().await {
            let fixed = fix_dzen_string(inp);
            let s = arg.get_builder().add(fixed).to_string();
            if let Err(_) = to_printer.send(Msg::Gen(id, s)) {
                return ExitReason::Error;
            }
        }
        ExitReason::Normal
    }
}
