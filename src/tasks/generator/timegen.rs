use async_trait::async_trait;
use chrono::prelude::*;
use super::{TimerGenerator,GenArg,Result};
use crate::dzen_format::DzenBuilder;

pub struct TimeGen{seconds: u32}

impl TimeGen {
    pub fn new() -> Self {
        TimeGen{seconds: 0}
    }
}

#[async_trait]
impl TimerGenerator for TimeGen {
    async fn update(&mut self, _name: &str) -> Result<String> {
        let now = Local::now();
        self.seconds = now.second();
        Ok(now.format("%a %Y-%m-%d %H:%M").to_string())
    }

    fn get_delay(&self, _arg: &Option<GenArg>) -> u64 {
        // NOTE: the +1 is for safety
        (60 - self.seconds as u64) + 1
    }
}
