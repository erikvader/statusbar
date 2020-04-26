use sysinfo::{SystemExt,ProcessorExt};
use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result};
use crate::dzen_format::DzenBuilder;

const LEVELS: &[(i32, &str)] = &[(50, "yellow"), (75, "red")];

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
            let mut bu = arg.get_builder();
            for p in self.sys.get_processors() {
                let usage = p.get_cpu_usage().round();
                bu = bu.add_not_empty("/")
                    .add(format!("{:0>2}", usage))
                    .color_step(usage as i32, LEVELS);
            }
            Ok(bu.name_click(1, name).to_string())
        } else {
            let usage = self.sys.get_global_processor_info().get_cpu_usage().round();

            Ok(arg.get_builder()
               .add(usage.to_string())
               .add("%")
               .color_step(usage as i32, LEVELS)
               .name_click(1, name)
               .to_string())
        }
    }

    async fn on_msg(&mut self, _msg: String) -> Result<bool> {
        self.detailed = !self.detailed;
        Ok(false)
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(2)
    }
}
