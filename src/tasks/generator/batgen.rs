use std::collections::HashSet;
use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use crate::dzen_format::DzenBuilder;
use std::path::PathBuf;
use tokio::fs;
use tokio::stream::StreamExt;

const HWMON: &str = "/sys/class/hwmon";

// /sys/class/power_supply/BAT0/{capacity,status}

pub struct TempGen {
    name: String,
    file: PathBuf
}

impl TempGen {
    pub fn new() -> Self {
        TempGen{
            name: "".to_string(),
            file: PathBuf::new(),
        }
    }
}

#[async_trait]
impl TimerGenerator for TempGen {
    async fn init(&mut self, arg: &Option<GenArg>) -> Result<()> {
        let argname = if let Some(GenArg{arg: Some(a), ..}) = arg {
            a
        } else {
            eprintln!("I need a name");
            return Err(ExitReason::Error);
        };

        let mut hwmons = fs::read_dir(HWMON).await?;
        while let Some(hw) = hwmons.next_entry().await? {
            let mut p = hw.path();
            p.push("name");
            let name = String::from_utf8(fs::read(p).await?)?;
            if name == *argname {
                
            }
        }
        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn display(&self, name: &str, arg: &Option<GenArg>) -> Result<String> {
        Ok("".to_string())
    }

    async fn on_msg(&mut self, msg: String) -> Result<bool> {
        Ok(false)
    }
}
