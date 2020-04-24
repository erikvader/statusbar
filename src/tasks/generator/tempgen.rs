use sysinfo::{SystemExt,ComponentExt,Component};
use std::collections::HashSet;
use async_trait::async_trait;
use std::path::Path;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use crate::dzen_format::DzenBuilder;
use crate::dzen_format::utils::bytes_to_ibibyte_string as byte_to_string;

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
    async fn init(&mut self, arg: &Option<GenArg>) -> Result<()> {
        self.sys.refresh_components_list();
        let avail_comps = self.sys.get_components()
            .into_iter()
            .map(|c| c.get_label())
            .collect::<HashSet<&str>>();

        if let Some(GenArg{arg: Some(a), ..}) = arg {
            if !avail_comps.contains(a.as_str()) {
                eprintln!("{} does not seem to be a proper temp thingy", a);
                eprintln!("you can choose from: {:?}", avail_comps);
                return Err(ExitReason::Error);
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

    fn display(&self, _name: &str, arg: &Option<GenArg>) -> Result<String> {
        let comp = self.sys.get_components()
            .into_iter()
            .find(|c| c.get_label() == self.name)
            .ok_or(ExitReason::Error)?;

        let temp = comp.get_temperature().trunc();
        // let max = comp.get_max();

        let o = DzenBuilder::from(&temp.to_string())
            .add("°C")
            .to_string();

        Ok(o)
    }

    async fn on_msg(&mut self, _msg: String) -> Result<bool> {
        Ok(false)
    }
}
