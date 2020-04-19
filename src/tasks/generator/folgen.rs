use tokio;
use tokio::process::Command;
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use super::*;
use crate::tasks::ExitReason;

pub struct FolGen;

impl FolGen {
    pub fn new() -> Self {
        FolGen
    }
}

#[async_trait]
impl Generator for FolGen {
    async fn start(
        &mut self,
        to_printer: broadcast::Sender<Msg>,
        mut from_pipo: mpsc::Receiver<String>,
        id: GenId,
        arg: Option<GenArg>,
        _name: String
    ) -> ExitReason
    {
        let cmd =
            if let Some(GenArg{arg: Some(cmd), ..}) = arg {
                cmd.to_string()
            } else {
                eprintln!("I want an command as argument");
                return ExitReason::Error;
            };

        let mut path = std::env::var("PATH").expect("couldn't get PATH");
        path.insert_str(0, ":");
        path.insert_str(0, crate::config::SCRIPT_PATH);
        if path.starts_with("~") {
            path.replace_range(..1, unsafe{&crate::HOME});
        }

        let proc = match Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .env("PATH", path)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    return ExitReason::Error;
                }
            };

        let mut stdout = BufReader::new(proc.stdout.unwrap()).lines();

        let reas = loop {
            let line = tokio::select! {
                l = stdout.next_line() => Some(l),
                x = from_pipo.recv() => {
                    match x {
                        None => break ExitReason::Normal,
                        Some(_) => None
                    }
                }
            };

            if let Some(x) = line {
                match x {
                    Ok(Some(l)) => {
                        if let Err(_) = to_printer.send((id, l)) {
                            break ExitReason::Error;
                        }
                    }
                    Ok(None) => {
                        eprintln!("process terminated");
                        break ExitReason::Error;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        break ExitReason::Error;
                    }
                }
            }
        };

        reas
    }
}
