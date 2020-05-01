use std::collections::HashMap;
use tokio;
use tokio::sync::broadcast::{self, RecvError};
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::process::Command;
use tokio::time::{self, Duration, Instant};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::config::*;
use crate::bar::*;
use crate::tasks::ExitReason;
use crate::tasks::generator::GenId;
use crate::dzen_format::DzenBuilder;
use super::Msg;

const ACC_DUR: Duration = Duration::from_millis(40);

fn spawn_dzen(xin: &str, al: &str, x: &str, w: &str) -> tokio::io::Result<tokio::process::Child> {
    let fg = crate::config::theme("fg").unwrap_or("#ffffff");
    let bg = crate::config::theme("bg").unwrap_or("#000000");
    Command::new("dzen2")
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .args(&["-fg", fg])
        .args(&["-bg", bg])
        .args(&["-fn", DZEN_FONT])
        .args(&["-h", "26"])
        .args(&["-xs", xin])
        .args(&["-ta", al])
        .args(&["-x", x])
        .args(&["-w", w])
        .args(&["-dock"])
        .args(&["-e", ""])
        .spawn()
}

fn build_side<'a,'b>(
    it: impl Iterator<Item = &'a GenId>,
    output: &'b HashMap<GenId, String>,
    sep: &'b str
) -> DzenBuilder<'b>
{
    it.map(|x| output.get(x).unwrap().as_str())
        .filter(|x| !x.is_empty())
        .fold(DzenBuilder::new(), |b, i| b % sep + i)
}

fn spawn_tray() -> tokio::io::Result<tokio::process::Child> {
    Command::new("trayer")
        .kill_on_drop(true)
        .args(&["--edge", "top",
                "--widthtype", "request",
                "--height", "16",
                "--distance", "5",
                // NOTE: this has a different way for specifying the
                // screen, and I don't see how to choose one from
                // xinerama index or output name, so this will always
                // be put on the primary screen (which is basically
                // always wanted).
                "--monitor", "primary"])
        .spawn()
}

pub async fn dzen_printer(mut recv: broadcast::Receiver<Msg>, config: BarConfig) -> ExitReason {
    // aliases
    let sep = config.get_separator();
    let pad = config.get_padding();

    // output buffer
    let mut output = HashMap::<GenId, String>::new();
    for id in config.iter() {
        output.insert(*id, "".to_string());
    }

    // spawn dzen on the right screen
    let bar_width = (config.get_screen_width() / 2).to_string();
    let xin = config.get_xinerama().to_string();
    let (dzenl, dzenr) = spawn_dzen(&xin, "l", "0", &bar_width)
        .and_then(|l| spawn_dzen(&xin, "r", &bar_width, &bar_width)
                        .and_then(|r| Ok((l, r))))
        .map_err(|e| {
            log::error!("couldn't spawn dzen '{}'", e);
            return ExitReason::Error;
        })
        .unwrap();

    // spawn tray
    let tray = Arc::new(Mutex::new(None));
    if config.wants_tray() {
        let tray2 = tray.clone();
        tokio::spawn(async move {
            time::delay_for(Duration::from_secs(2)).await;
            let t = spawn_tray()
                .map_err(|e| {
                    log::warn!("coudln't spawn trayer '{}'", e);
                })
                .ok();
            *tray2.lock().await = t;
        });
    }

    let mut lstdin = dzenl.stdin.unwrap();
    let mut rstdin = dzenr.stdin.unwrap();

    let mut delay = time::delay_for(ACC_DUR);
    let mut waiting = false;
    // receive new strings to output buffer and occasionally print
    // them to dzen
    loop {
        // accumulate close changes as one (`ACC_DUR` time from first message)
        select! {
            _    = &mut delay, if waiting => (),
            recv = recv.recv() =>
                match recv {
                    Err(RecvError::Lagged(_)) => continue,
                    Err(_) => break ExitReason::Normal,
                    Ok(Msg::Gen(id, msg)) => {
                        if output.contains_key(&id) {
                            *output.get_mut(&id).unwrap() = msg;

                            if !waiting {
                                delay.reset(Instant::now() + ACC_DUR);
                                waiting = true;
                            }
                        }
                        continue;
                    },
                    Ok(Msg::Tray) => {
                        if config.wants_tray() {
                            let tray2 = tray.clone();
                            tokio::spawn(async move {
                                let mut l = tray2.lock().await;
                                if let Some(c) = l.as_mut() {
                                    if let Err(e) = c.kill() {
                                        log::warn!("couldn't kill tray: {}", e);
                                    }
                                    if let Err(e) = c.await {
                                        log::warn!("couldn't await tray: {}", e);
                                    }
                                }
                                let t = spawn_tray()
                                    .map_err(|e| {
                                        log::warn!("coudln't spawn trayer '{}'", e);
                                    })
                                    .ok();
                                *l = t;
                            });
                        }
                    }
                }
        }
        waiting = false;

        // print to dzen
        let left_side = build_side(config.iter_left(), &output, sep)
            .lpad(pad)
            .to_stringln();

        let right_side = build_side(config.iter_right(), &output, sep)
            .rpad(pad)
            .to_stringln();

        let res = tokio::try_join!(
            lstdin.write_all(left_side.as_bytes()),
            rstdin.write_all(right_side.as_bytes())
        );

        if let Err(e) = res {
            log::error!("couldn't write to dzen '{}'", e);
            return ExitReason::Error;
        }
    }
}
