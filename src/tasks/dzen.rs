use std::collections::HashMap;
// use itertools::Itertools;
use tokio;
use tokio::sync::broadcast::{self, RecvError};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use crate::config::*;
use crate::bar::*;
use crate::tasks::ExitReason;
use crate::tasks::generator::GenId;
use crate::dzen_format::DzenBuilder;
use super::Msg;

fn spawn_dzen(xin: &str, al: &str, x: &str, w: &str) -> tokio::io::Result<tokio::process::Child> {
    Command::new("dzen2")
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .args(&["-fg", "#dfdfdf"])
        .args(&["-bg", "#333333"])
        .args(&["-fn", DZEN_FONT])
        .args(&["-h", "26"])
        .args(&["-xs", xin])
        .args(&["-ta", al])
        .args(&["-x", x])
        .args(&["-w", w])
        .spawn()
}

fn build_side<'a,'b>(it: impl Iterator<Item = &'a GenId>, output: &'b HashMap<GenId, String>, sep: &'b str) -> DzenBuilder<'b> {
    it
        .map(|x| output.get(x).unwrap().as_str())
        .filter(|x| !x.is_empty())
        .fold(DzenBuilder::new(), |b, i| b % sep + i)
}

pub async fn dzen_printer(mut recv: broadcast::Receiver<Msg>, config: BarConfig) -> ExitReason {
    let sep = config.get_separator();
    let pad = config.get_padding().to_string();

    let mut output = HashMap::<GenId, String>::new();
    for id in config.iter() {
        output.insert(*id, "".to_string());
    }

    let bar_width = (config.get_screen_width() / 2).to_string();
    let xin = config.get_xinerama().to_string();
    let (dzenl, dzenr) = spawn_dzen(&xin, "l", "0", &bar_width)
        .and_then(|l| spawn_dzen(&xin, "r", &bar_width, &bar_width)
                  .and_then(|r| Ok((l, r))))
        .map_err(|e| {
            eprintln!("couldn't spawn dzen '{}'", e);
            return ExitReason::Error;
        })
        .unwrap();

    let mut lstdin = dzenl.stdin.unwrap();
    let mut rstdin = dzenr.stdin.unwrap();

    // TODO: accumulate close updates into one
    loop {
        match recv.recv().await {
            Err(RecvError::Lagged(_)) => continue,
            Err(_) => break ExitReason::Normal,
            Ok((id, msg)) => {
                if !output.contains_key(&id) {
                    continue;
                }

                *output.get_mut(&id).unwrap() = msg;

                let left_side = build_side(config.iter_left(), &output, sep)
                    .lpad(&pad)
                    .to_stringln();

                let right_side = build_side(config.iter_right(), &output, sep)
                    .rpad(&pad)
                    .to_stringln();

                let res = tokio::try_join!(
                    lstdin.write_all(left_side.as_bytes()),
                    rstdin.write_all(right_side.as_bytes())
                );

                if let Err(e) = res {
                    eprintln!("couldn't write to dzen '{}'", e);
                    return ExitReason::Error;
                }
            }
        }
    }
}
