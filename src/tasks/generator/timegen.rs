use async_trait::async_trait;
use chrono::prelude::*;
use super::{TimerGenerator,GenArg,Result};
use crate::dzen_format::DzenBuilder;

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

    fn display(&self, _name: &str) -> Result<String> {
        Ok(self.datetime.format("%a %Y-%m-%d %H:%M").to_string())
    }

    fn get_delay(&self, _arg: &Option<GenArg>) -> u64 {
        // NOTE: the +1 is for safety
        (60 - self.datetime.second() as u64) + 1
    }
}
