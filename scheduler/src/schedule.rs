use cron;
use humantime::parse_duration;
use std::time::Instant;
use std::str::FromStr;
use chrono::{DateTime, Utc, Timelike};

#[derive(Debug)]
pub enum Schedule {
    Cron(cron::Schedule, DateTime<Utc>),
    Interval(std::time::Duration, Instant),
}

impl Schedule {
    
    pub fn from_str(raw: &str) -> Result<Schedule, Box<dyn std::error::Error>> {
        if let Some(interval_str) = raw.strip_prefix("@every ") {
            let duration = parse_duration(interval_str)?;
            Ok(Schedule::Interval(duration, Instant::now() + duration))
        } else {
            let schedule = cron::Schedule::from_str(raw.trim())?;
            let next_run = schedule.upcoming(Utc).next().unwrap();
            Ok(Schedule::Cron(schedule, next_run))
        }
    }

    pub fn should_run(&mut self) -> bool {
        match self {
            Schedule::Cron(schedule, next_run) => {
                let now = Utc::now().with_nanosecond(0).unwrap();
                if *next_run <= now {
                    *next_run = schedule.upcoming(Utc).next().unwrap();
                    true
                } else {
                    false
                }
            }
            Schedule::Interval(duration, next_run) => {
                let now = Instant::now();
                if *next_run <= now {
                    *next_run = now + *duration;
                    true
                } else {
                    false
                }
            }
        }
    }

}
