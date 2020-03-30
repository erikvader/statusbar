use sysinfo::{SystemExt,ProcessorExt};
use itertools::Itertools;
use async_trait::async_trait;
use super::TimerGenerator;

pub struct CpuGen{sys: sysinfo::System, detailed: bool}

impl CpuGen {
    pub fn new() -> Self {
        CpuGen{sys: sysinfo::System::new(), detailed: false}
    }
}

#[async_trait]
impl TimerGenerator for CpuGen {
    fn get_delay(&self) -> u64 { 3 }

    async fn init(&mut self, arg: Option<String>) {
        if let Some(a) = arg {
            if a == "detailed" {
                self.detailed = true;
            }
        }
    }

    async fn update(&mut self) -> String {
        self.sys.refresh_cpu();
        if self.detailed {
            self.sys.get_processors()
                .iter()
                .map(|p| format!("{:0>2}", p.get_cpu_usage().round()))
                .join("/")
        } else {
            self.sys.get_global_processor_info().get_cpu_usage().round().to_string()
        }
    }

    async fn on_msg(&mut self, _msg: String) {
        self.detailed = !self.detailed;
    }
}
