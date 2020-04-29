use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use std::collections::HashMap;
use tokio;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::sync::oneshot;
use crate::bar::*;
use super::ExitReason;
use super::generator::{genid_to_generator,GenArg};
use super::dzen::dzen_printer;
use super::pipo::pipo_reader;

const MPSC_SIZE: usize = 32;

pub async fn main(mut setup: SetupConfig) -> ExitReason {
    let mut tasks = FuturesUnordered::<JoinHandle<ExitReason>>::new();

    let mut pipo_map = HashMap::new();

    {
        let (broad_send, _) = broadcast::channel(MPSC_SIZE);

        let mut args = setup.extract_args();
        for g in setup.iter() {
            let (pipo_send, pipo_recv) = mpsc::channel(MPSC_SIZE);
            let bs = broad_send.clone();
            let a = args.remove(g).unwrap_or(GenArg::empty());
            let gg = *g;
            let name = setup.get_name(*g).cloned().unwrap_or(g.to_string());
            let name2 = name.clone();
            tasks.push(tokio::spawn(async move {
                let mut gen = genid_to_generator(gg);
                gen.start(bs, pipo_recv, gg, a, name2).await
            }));
            if let Some(_) = pipo_map.insert(name, pipo_send) {
                panic!("some generators have the same name!");
            }
        }

        for b in setup.take_bars().into_iter() {
            tasks.push(tokio::spawn(dzen_printer(broad_send.subscribe(), b)));
        }
    }

    let mut shutdown_pipo = Some({
        let (sp, pipo_shutdown_recv) = oneshot::channel();
        tasks.push(tokio::spawn(pipo_reader(pipo_map, pipo_shutdown_recv)));
        sp
    });

    let mut reason = ExitReason::Normal;
    while let Some(res_r) = tasks.next().await {
        if let Err(_) = res_r {
            log::warn!("coudln't join??");
            continue;
        }
        shutdown_pipo = shutdown_pipo.and_then(|p| p.send(()).ok()).and(None);
        let r = res_r.unwrap();
        reason = reason.combine(r);
    }

    log::info!("all tasks have finished, exiting...");
    reason
}
