use tokio;
use tokio::process::Command;
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use super::*;
use crate::tasks::ExitReason;
use crate::dzen_format::external::fix_dzen_string;

pub struct OneGen;

impl OneGen {
    pub fn new() -> Self {
        OneGen
    }
}

// TODO: move this to a more sensible location
pub fn spawn(cmd: &str, first: bool) -> std::io::Result<tokio::process::Child> {
    let mut path = std::env::var("PATH").expect("couldn't get PATH");
    path.insert_str(0, ":");
    path.insert_str(0, crate::config::SCRIPT_PATH);
    if path.starts_with("~") {
        path.replace_range(..1, unsafe{&crate::HOME});
    }

    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .env("PATH", path)
        .env("STS_INIT", if first {"yes"} else {""})
        .kill_on_drop(true)
        .stdout(std::process::Stdio::piped())
        .spawn()
}

#[async_trait]
impl Generator for OneGen {
    async fn start(
        &mut self,
        to_printer: broadcast::Sender<Msg>,
        mut from_pipo: mpsc::Receiver<String>,
        id: GenId,
        arg: GenArg,
        name: String
    ) -> ExitReason
    {
        let cmd =
            if let Some(cmd) = &arg.arg {
                cmd.to_string()
            } else {
                log::error!("I want a command as argument");
                return ExitReason::Error;
            };

        let mut run_cmd = true;
        let mut proc = match spawn(&cmd, true) {
            Ok(c) => c,
            Err(e) => {
                log::error!("{}", e);
                return ExitReason::Error;
            }
        };
        let mut sout = BufReader::new(proc.stdout.unwrap()).lines();

        let reas = loop {
            let line = tokio::select! {
                out = sout.next_line(), if run_cmd => {
                    Some(out)
                }
                x = from_pipo.recv() => {
                    match x {
                        None => break ExitReason::Normal,
                        Some(_) if !run_cmd => {
                            proc = match spawn(&cmd, false) {
                                Ok(c) => c,
                                Err(e) => {
                                    log::error!("{}", e);
                                    break ExitReason::Error;
                                }
                            };
                            sout = BufReader::new(proc.stdout.unwrap()).lines();
                            run_cmd = true;
                            None
                        }
                        Some(_) => None
                    }
                }
            };

            if let Some(l) = line {
                match l {
                    Ok(Some(x)) => {
                        let fixed = fix_dzen_string(x);
                        let clicked = arg.get_builder()
                            .add(fixed)
                            .name_click(1, &name)
                            .to_string();
                        if let Err(_) = to_printer.send(Msg::Gen(id, clicked)) {
                            break ExitReason::Error;
                        }
                    }
                    Ok(None) => {
                        run_cmd = false;
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        break ExitReason::Error;
                    }
                }
            }
        };

        reas
    }
}
