mod tasks;
mod bar;
mod dzen_format;
mod x;
mod config;
mod kill;

use tokio;
use core::time::Duration;

use tasks::main_task;

use stderrlog as SL;

// TODO: få start att acceptera en lista utav GenArgs där alla måste
// vara likadana förutom prepend som får (borde) vara annorlunda.
// Borde få en lista utav GenId också? Name borde vara lika på alla.
// Detta för att kunna skapa en generator som skickar sin output till
// flera olika ställen med olika prepend. najs om man vill visa tiden
// på olika skärmar, alla med olika ikoner, utan att behöva spawna en
// annars identisk generator flera gånger.
// TODO: använd spawn_local med tanke på att det bara är en thread (basic scheduler)
// TODO: byt ut named pipe till sockets, eller kanske ha båda?
// TODO: kunna ändra antalet dzen utan att starta om allting. Typ när
// en ny skärm kommer in i bilden.

fn main() {
    let reason = {
        SL::new()
            .module(std::module_path!())
            .timestamp(SL::Timestamp::Second)
            .show_level(true)
            .color(SL::ColorChoice::Auto)
            .verbosity(2) // 4 = trace, 0 = error
            .init()
            .expect("couldn't start logger");

        let setup = config::config().build().unwrap();

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

        reason
    };

    std::process::exit(reason.get_exit_code());
}
