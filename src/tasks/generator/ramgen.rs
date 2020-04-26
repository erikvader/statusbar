use sysinfo::SystemExt;
use async_trait::async_trait;
use super::{Result,TimerGenerator,GenArg};

const LEVELS: &[(i32, &str)] = &[(60, "yellow"), (80, "red")];

pub struct RamGen{sys: sysinfo::System}

impl RamGen {
    pub fn new() -> Self {
        RamGen{sys: sysinfo::System::new()}
    }
}

#[async_trait]
impl TimerGenerator for RamGen {
    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_memory();
        Ok(())
    }

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let usage = ((self.sys.get_used_memory() as f64 / self.sys.get_total_memory() as f64) * 100.0).round();

        let mut bu = arg.get_builder()
            .add(usage.to_string())
            .add("%")
            .color_step(usage as i32, LEVELS);

        let swap = self.sys.get_used_swap();
        if swap > 0 {
            let total_swap = self.sys.get_total_swap() as f64;
            let perc = ((swap as f64 / total_swap) * 100.0).round();
            bu = bu.add(" (")
                .new_section()
                .add(perc.to_string())
                .color_step(perc as i32, LEVELS)
                .add(")");
        }

        Ok(bu.to_string())
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(2)
    }
}



