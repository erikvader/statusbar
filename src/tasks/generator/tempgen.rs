use sysinfo::{SystemExt,ComponentExt};
use std::collections::HashSet;
use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result,ExitReason};

const LEVELS: &[(i32, &str)] = &[(50, "yellow"), (70, "red")];

pub struct TempGen {
    sys: sysinfo::System,
    name: String
}

impl TempGen {
    pub fn new() -> Self {
        TempGen{
            sys: sysinfo::System::new(),
            name: "".to_string(),
        }
    }
}

#[async_trait]
impl TimerGenerator for TempGen {
    async fn init(&mut self, arg: &GenArg) -> Result<()> {
        self.sys.refresh_components_list();
        let avail_comps = self.sys.get_components()
            .into_iter()
            .map(|c| c.get_label())
            .collect::<HashSet<&str>>();

        if let Some(a) = &arg.arg {
            if !avail_comps.contains(a.as_str()) {
                log::warn!("{} does not seem to be a proper temp thingy", a);
                log::warn!("you can choose from: {:?}", avail_comps);
                return Err(ExitReason::NonFatal);
            } else {
                self.name = a.to_string();
            }
        }

        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        self.sys.refresh_components();
        Ok(())
    }

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let comp = self.sys.get_components()
            .into_iter()
            .find(|c| c.get_label() == self.name)
            .ok_or(ExitReason::Error)?;

        let temp = comp.get_temperature().trunc();
        // let max = comp.get_max();

        let o = arg.get_builder()
            .add(temp.to_string())
            .add("°C")
            .color_step(temp as i32, LEVELS)
            .to_string();

        Ok(o)
    }

    async fn on_msg(&mut self, _msg: String) -> Result<bool> {
        Ok(false)
    }

    fn get_delay(&self, arg: &GenArg) -> u64 {
        arg.timeout.unwrap_or(2)
    }
}
