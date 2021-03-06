use async_trait::async_trait;
use super::{TimerGenerator,GenArg,Result,ExitReason};
use std::path::Path;
use tokio::fs;

// TODO: använd udev för att lyssna på om den börjar ladda elr inte.

const CAP_FILE:    &str = "/sys/class/power_supply/BAT0/capacity";
const STATUS_FILE: &str = "/sys/class/power_supply/BAT0/status";

pub struct BatGen {
    capacity: u8,
    charging: bool,
}

impl BatGen {
    pub fn new() -> Self {
        BatGen{
            capacity: 0,
            charging: false,
        }
    }
}

#[async_trait]
impl TimerGenerator for BatGen {
    async fn init(&mut self, _arg: &GenArg) -> Result<()> {
        if !Path::new(CAP_FILE).exists() {
            log::warn!("couldn't find a battery");
            return Err(ExitReason::NonFatal);
        } else {
        }
        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        let cap = fs::read_to_string(CAP_FILE).await?;
        self.capacity = match cap.trim_end().parse() {
            Ok(c) => c,
            Err(_) => {
                log::warn!("couldn't parse capacity");
                std::u8::MAX
            }
        };

        self.charging = match fs::read_to_string(STATUS_FILE).await?.as_str().trim_end() {
            "Charging" => true,
            "Full" => true,
            _ => false
        };

        Ok(())
    }

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let mut s = arg.get_builder()
            .add(self.capacity.to_string())
            .add("%");

        if self.charging {
            s = s.colorize("green");
        } else {
            s = s.color_step(self.capacity as i32, &[(0, "red"), (16, "yellow"), (30, "fg")])
        }

        Ok(s.to_string())
    }
}
