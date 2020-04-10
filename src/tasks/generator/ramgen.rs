use sysinfo::SystemExt;
use async_trait::async_trait;
use super::{Result,TimerGenerator};

pub struct RamGen{sys: sysinfo::System}

impl RamGen {
    pub fn new() -> Self {
        RamGen{sys: sysinfo::System::new()}
    }
}

#[async_trait]
impl TimerGenerator for RamGen {
    async fn update(&mut self, _name: &str) -> Result<String> {
        self.sys.refresh_memory();
        let usage = ((self.sys.get_used_memory() as f64 / self.sys.get_total_memory() as f64) * 100.0).round();
        let swap = self.sys.get_used_swap();
        let swap_str = if swap > 0 {
            let total_swap = self.sys.get_total_swap() as f64;
            let perc = ((swap as f64 / total_swap) * 100.0).round();
            format!(" ({}%)", perc)
        } else {
            "".to_string()
        };
        Ok(format!("{}%{}", usage, swap_str))
    }
}



