use sysinfo::{SystemExt,ProcessorExt};
use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result};
use crate::dzen_format::DzenBuilder;

pub struct CpuGen{sys: sysinfo::System, detailed: bool}

impl CpuGen {
    pub fn new() -> Self {
        CpuGen{sys: sysinfo::System::new(), detailed: false}
    }
}

#[async_trait]
impl TimerGenerator for CpuGen {
    async fn init(&mut self, arg: &GenArg) -> Result<()> {
        if let Some(a) = &arg.arg {
            if a == "detailed" {
                self.detailed = true;
            }
        }
        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_cpu();
        Ok(())
    }

    fn display(&self, name: &str, arg: &GenArg) -> Result<String> {
        if self.detailed {
            let ps = self.sys.get_processors()
                .iter()
                .map(|p| format!("{:0>2}", p.get_cpu_usage().round()))
                .collect::<Vec<String>>();
            let str_ps = ps.iter().map(|s| s.as_str());
            let mut b = DzenBuilder::new();
            for p in str_ps {
                b = b % "|" + p;
            }
            Ok(b.name_click(1, name).to_string())
        } else {
            let usage = self.sys.get_global_processor_info().get_cpu_usage().round().to_string();
            Ok(arg.get_builder()
               .add(usage)
               .add("%")
               .name_click(1, name)
               .to_string())
        }
    }

    async fn on_msg(&mut self, _msg: String) -> Result<bool> {
        self.detailed = !self.detailed;
        Ok(false)
    }
}
