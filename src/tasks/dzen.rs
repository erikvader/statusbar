use std::collections::HashMap;
use tokio;
use tokio::sync::broadcast::{self, RecvError};
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::process::Command;
use tokio::time::{self, Duration, Instant};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::kill::*;
use crate::config::*;
use crate::bar::*;
use crate::tasks::ExitReason;
use crate::tasks::generator::GenId;
use crate::dzen_format::DzenBuilder;
use super::Msg;

const ACC_DUR: Duration = Duration::from_millis(40);

fn spawn_dzen(xin: &str, al: &str, x: u16, w: u16) -> tokio::io::Result<ChildTerminator> {
    let fg = crate::config::theme("fg").unwrap_or("#ffffff");
    let bg = crate::config::theme("bg").unwrap_or("#000000");
    Command::new("dzen2")
        .kill_on_drop(false)
        .stdin(std::process::Stdio::piped())
        .args(&["-fg", fg])
        .args(&["-bg", bg])
        .args(&["-fn", DZEN_FONT])
        .args(&["-h", "26"])
        .args(&["-xs", xin])
        .args(&["-ta", al])
        .args(&["-x", &x.to_string()])
        .args(&["-w", &w.to_string()])
        .args(&["-dock"])
        .args(&["-e", ""])
        .spawn()
        .map(|c| ChildTerminator::new(c))
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

fn spawn_tray(secs: u64, p: Arc<Mutex<Option<ChildTerminator>>>) {
    tokio::spawn(async move {
        let mut l = match p.try_lock() {
            Ok(lock) => lock,
            Err(_) => {
                log::info!("tray lock already taken, ignoring...");
                return;
            }
        };

        time::delay_for(Duration::from_secs(secs)).await;

        if let Some(c) = l.as_mut() {
            if let Err(e) = c.terminate() {
                log::warn!("couldn't terminate tray '{}'", e);
            }

            if let Err(e) = c.await {
                log::warn!("couldn't await because '{}'", e);
            }
        }

        let t = Command::new("trayer")
            .kill_on_drop(false)
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
            .map(|c| ChildTerminator::new(c))
            .map_err(|e| {
                log::warn!("coudln't spawn trayer '{}'", e);
            })
            .ok();

        *l = t;
    });
}

pub async fn dzen_printer(mut recv: broadcast::Receiver<Msg>, config: BarConfig) -> ExitReason {
    // aliases
    let sep = config.get_separator();
    let pad = config.get_padding();

    // output buffer
    let mut output = HashMap::<GenId, String>::new();
    for id in config.iter() {
        output.insert(*id, "xxx".to_string());
    }

    // spawn dzen on the right screen
    // let bar_width = (config.get_screen_width() / 2).to_string();
    let left_bar_width = ((config.get_screen_width() as f32) * config.get_split()) as u16;
    let right_bar_width = config.get_screen_width() - left_bar_width;
    let xin = config.get_xinerama().to_string();

    let tmp = spawn_dzen(&xin, "l", 0, left_bar_width)
        .and_then(|l| spawn_dzen(&xin, "r", left_bar_width, right_bar_width)
                  .and_then(|r| Ok((l, r))));

    let (mut dzenl, mut dzenr) = match tmp {
        Ok(dldr) => dldr,
        Err(e) => {
            log::error!("couldn't spawn dzen '{}'", e);
            return ExitReason::Error;
        }
    };

    // spawn tray
    let tray = Arc::new(Mutex::new(None));
    if config.wants_tray() {
        spawn_tray(2, tray.clone());
    }

    let lstdin = dzenl.as_mut_ref().stdin.as_mut().unwrap();
    let rstdin = dzenr.as_mut_ref().stdin.as_mut().unwrap();

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
                            spawn_tray(0, tray.clone());
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
            break ExitReason::Error;
        }
    }
}
