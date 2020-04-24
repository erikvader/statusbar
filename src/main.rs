mod tasks;
mod bar;
mod dzen_format;
mod x;
mod config;

use tokio;
use core::time::Duration;

use tasks::*;
use tasks::main_task;

// TODO: resten utav generatorer
// - battery
// TODO: en theme för ikoner också.
// TODO: fixa presentationen utav alla generators

pub static mut HOME: String = String::new();

fn main() {
    let h = std::env::var("HOME").expect("couldn't get HOME");
    unsafe {HOME = h;}

    let setup = config::config().unwrap();

    // NOTE: explicitly creating and shutting down a runtime like this
    // is required because of https://github.com/tokio-rs/tokio/issues/2318
    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    let reason = runtime.block_on(main_task::main(setup));

    // NOTE: a non-zero timeout shouldn't be needed because we should
    // exit _only_ if all tasks have already exited, but just to be
    // safe
    runtime.shutdown_timeout(Duration::from_secs(1));

    std::process::exit(match reason {
        ExitReason::Normal => 0,
        ExitReason::Error  => 1,
        ExitReason::Signal => 2, // TODO: should shutdown from signals return non-zero exit codes?
        ExitReason::SignalError => 3
    });
}
