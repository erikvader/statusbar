use async_trait::async_trait;
use chrono::prelude::*;
use super::{TimerGenerator,GenArg,Result};

pub struct TimeGen{datetime: DateTime<Local>}

impl TimeGen {
    pub fn new() -> Self {
        TimeGen{datetime: Local::now()}
    }
}

#[async_trait]
impl TimerGenerator for TimeGen {
    async fn update(&mut self) -> Result<()> {
        self.datetime = Local::now();
        Ok(())
    }

    fn display(&self, _name: &str, arg: &GenArg) -> Result<String> {
        let d = match self.datetime.weekday() {
            Weekday::Mon => "Mån",
            Weekday::Tue => "Tis",
            Weekday::Wed => "Ons",
            Weekday::Thu => "Tor",
            Weekday::Fri => "Fre",
            Weekday::Sat => "Lör",
            Weekday::Sun => "Sön",
        };

        let s = arg.get_builder()
            .add(d)
            .add(" ")
            .add(self.datetime.format("%Y-%m-%d %H:%M").to_string())
            .to_string();
        Ok(s)
    }

    fn get_delay(&self, _arg: &GenArg) -> u64 {
        // NOTE: the +1 is for safety
        (60 - self.datetime.second() as u64) + 1
    }
}
