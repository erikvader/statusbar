use sysinfo::SystemExt;
use async_trait::async_trait;
use super::TimerGenerator;

pub struct RamGen{sys: sysinfo::System}

impl RamGen {
    pub fn new() -> Self {
        RamGen{sys: sysinfo::System::new()}
    }
}

#[async_trait]
impl TimerGenerator for RamGen {
    fn get_delay(&self) -> u64 { 7 }
    async fn update(&mut self, _name: &str) -> String {
        self.sys.refresh_memory();
        let usage = ((self.sys.get_used_memory() as f64 / self.sys.get_total_memory() as f64) * 100.0).round();
        format!("{}%", usage)
    }
}



