mod tasks;
mod bar;
mod dzen_format;
mod x;
mod config;
mod kill;

use tokio;
use core::time::Duration;

use tasks::*;
use tasks::main_task;

use stderrlog as SL;

// TODO: stoppa in name i GenArg?
// TODO: få start att acceptera en lista utav GenArgs där alla måste
// vara likadana förutom prepend som får (borde) vara annorlunda.
// Borde få en lista utav GenId också? Name borde vara lika på alla.
// Detta för att kunna skapa en generator som skickar sin output till
// flera olika ställen med olika prepend. najs om man vill visa tiden
// på olika skärmar, alla med olika ikoner, utan att behöva spawna en
// annars identisk generator flera gånger.
// TODO: använd spawn_local med tanke på att det bara är en thread (basic scheduler)
// TODO: byt ut named pipe till sockets, eller kanske ha båda?

pub static mut HOME: String = String::new();

fn main() {
    let h = std::env::var("HOME").expect("couldn't get HOME");
    unsafe {HOME = h;}

    SL::new()
        .module(std::module_path!())
        .timestamp(SL::Timestamp::Second)
        .show_level(true)
        .color(SL::ColorChoice::Auto)
        .verbosity(2) // 4 = trace, 0 = error
        .init()
        .expect("couldn't start logger");

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
