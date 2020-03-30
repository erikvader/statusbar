mod tasks;
mod bar;
mod constants;
mod dzen_format;

use tokio;
use core::time::Duration;

use tasks::generator::*;
use tasks::*;
use tasks::main_task;

// TODO: göra en trait kanske för att skapa stränger att skicka till dzen?
// TODO: tänk på hur programmet ska startas. Alltid daemoniza? Hantera SIGHUP?

fn main() {
    let mut setup = bar::SetupConfig::new();
    let mut bar1 = bar::BarConfig::new(1);
    let c = setup.create_module(GenType::CPU, Some("detailed".to_string()));
    setup.name_module(c, "cpu".to_string());
    bar1.add_left(c);
    bar1.add_right(setup.create_module(GenType::RAM, None));
    setup.add_bar(bar1);

    let mut bar2 = bar::BarConfig::new(2);
    let echo = setup.create_module(GenType::ECHO, None);
    setup.name_module(echo, "adina".to_string());
    bar2.add_left(echo);
    setup.add_bar(bar2);

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
