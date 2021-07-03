use async_trait::async_trait;
use chrono::prelude::*;
use std::time::Instant;
use super::{TimerGenerator,GenArg,Result};

struct Timer {
    start: Instant,
    now: Instant,
}

pub struct TimeGen {
    datetime: DateTime<Local>,
    timer: Option<Timer>,
}

impl TimeGen {
    pub fn new() -> Self {
        TimeGen{
            datetime: Local::now(),
            timer: None,
        }
    }
}

#[async_trait]
impl TimerGenerator for TimeGen {
    async fn update(&mut self) -> Result<()> {
        self.datetime = Local::now();
        if let Some(t) = &mut self.timer {
            t.now = Instant::now();
        }
        Ok(())
    }

    fn display(&self, name: &str, arg: &GenArg) -> Result<String> {
        let d = match self.datetime.weekday() {
            Weekday::Mon => "Mån",
            Weekday::Tue => "Tis",
            Weekday::Wed => "Ons",
            Weekday::Thu => "Tor",
            Weekday::Fri => "Fre",
            Weekday::Sat => "Lör",
            Weekday::Sun => "Sön",
        };

        let mut s = arg.get_builder()
            .add(d)
            .add(" ")
            .add(self.datetime.format("%Y-%m-%d").to_string())
            .add(" ");

        if let Some(t) = &self.timer {
            let dur = (t.now - t.start).as_secs();
            s = s.new_section()
                .add(format!("{:02}:{:02}", dur / 60, dur % 60))
                .colorize("green")
                .everything();
        } else {
            s = s.add(self.datetime.format("%H:%M").to_string())
        }

        Ok(s.name_click(1, name)
           .to_string())
    }

    async fn on_msg(&mut self, msg: String) -> Result<bool> {
        if msg == "click 1" {
            if self.timer.is_some() {
                self.timer = None;
            } else {
                let n = Instant::now();
                self.timer = Some(Timer {start: n, now: n});
            }
            return Ok(true);
        } else if msg == "update" {
            return Ok(true);
        }
        Ok(false)
    }

    fn get_delay(&self, _arg: &GenArg) -> u64 {
        if self.timer.is_some() {
            1
        } else {
            // NOTE: the +1 is for safety
            (60 - self.datetime.second() as u64) + 1
        }
    }
}
