use nix::unistd::{mkfifo, unlink};
use nix::sys::stat;
use nix::Error::Sys;
use nix::errno::Errno::EEXIST;
use std::collections::HashMap;
use tokio;
use tokio::sync::mpsc;
use tokio::select;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use crate::constants::*;
use crate::tasks::ExitReason;

pub async fn pipo_reader(mut gens: HashMap<String, mpsc::Sender<String>>, shutdown: oneshot::Receiver<()>) -> ExitReason {
    // create pipe
    match mkfifo(FIFO_PATH, stat::Mode::S_IRWXU) {
        Ok(()) => (),
        Err(Sys(errno)) if errno == EEXIST => (),
        Err(e) => {
            eprintln!("couldn't create pipo at {} because '{}'", FIFO_PATH, e);
            return ExitReason::Error;
        }
    };

    let mut int_stream = signal(SignalKind::interrupt()).unwrap();
    let mut term_stream = signal(SignalKind::terminate()).unwrap();

    // read pipe
    let main_loop = async {
        'outer: loop {
            let mut reader = match File::open(FIFO_PATH).await {
                Ok(f) => BufReader::new(f),
                Err(e) => {
                    eprintln!("couldn't open pipe '{}' because '{}'", FIFO_PATH, e);
                    return ExitReason::Error;
                }
            };

            let mut buf = String::new();
            while let Ok(c) = reader.read_line(&mut buf).await {
                if c == 0 {
                    break;
                }
                let content = buf.trim_end();

                let (gid, msg) =
                    match content.match_indices(" ").next() {
                        Some((i, _)) => (&content[..i], &content[i+1..]),
                        _ => (content, "")
                    };

                if gid == "EXIT" {
                    break 'outer ExitReason::Normal;
                }

                if let Some(send) = gens.get_mut(gid) {
                    match send.try_send(msg.to_string()) {
                        Err(mpsc::error::TrySendError::Closed(_)) => break 'outer ExitReason::Error,
                        _ => ()
                    }
                }
                buf.clear();
            }
        }
    };

    let reason = select! {
        _ = int_stream.recv() => ExitReason::Signal,
        _ = term_stream.recv() => ExitReason::Signal,
        r = main_loop => r,
        _ = shutdown => ExitReason::Error
    };

    // remove pipe
    if let Err(e) = unlink(FIFO_PATH) {
        eprintln!("Couldn't remove pipe at {} because '{}'", FIFO_PATH, e);
    }

    reason
}
