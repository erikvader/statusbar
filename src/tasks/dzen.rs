use std::collections::HashMap;
use tokio;
use tokio::sync::broadcast::{self, RecvError};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use crate::constants::*;
use crate::bar::*;
use crate::tasks::ExitReason;
use crate::tasks::generator::GenId;

pub async fn dzen_printer(mut recv: broadcast::Receiver<Msg>, config: BarConfig) -> ExitReason {
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
        .args(&["-xs", &config.get_xinerama().to_string()])
        .spawn()
        .map_err(|e| {
            eprintln!("couldn't spawn dzen '{}'", e);
            return ExitReason::Error;
        })
        .unwrap();

    let mut stdin = dzen.stdin.unwrap();

    loop {
        match recv.recv().await {
            Err(RecvError::Lagged(_)) => continue,
            Err(_) => break ExitReason::Normal,
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
                        buf.push_str(config.get_separator());
                    }
                    buf.push_str(x);
                    first = false;
                }
                buf.push_str("\n");
                if let Err(e) = stdin.write_all(buf.as_bytes()).await {
                    eprintln!("couldn't write to dzen '{}'", e);
                    return ExitReason::Error;
                }
            }
        }
    }
}
