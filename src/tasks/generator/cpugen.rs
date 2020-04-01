use sysinfo::{SystemExt,ProcessorExt};
use itertools::Itertools;
use async_trait::async_trait;
use super::TimerGenerator;
use crate::dzen_format::DzenBuilder;

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

    async fn update(&mut self, name: &str) -> String {
        self.sys.refresh_cpu();
        if self.detailed {
            let ps = self.sys.get_processors()
                .iter()
                .map(|p| format!("{:0>2}", p.get_cpu_usage().round()))
                .collect::<Vec<String>>();
            let str_ps = ps.iter().map(|s| s.as_str());
            let mut b = DzenBuilder::new();
            for p in str_ps {
                b = b % "|" + DzenBuilder::from(p);
            }
            b.name_click("1", name).to_string()
        } else {
            let usage = self.sys.get_global_processor_info().get_cpu_usage().round().to_string();
            DzenBuilder::from(usage.as_str())
                .add("%")
                .name_click("1", name)
                .to_string()
        }
    }

    async fn on_msg(&mut self, _msg: String) {
        self.detailed = !self.detailed;
    }
}
