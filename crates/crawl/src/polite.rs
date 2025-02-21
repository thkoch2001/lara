use simple_moving_average::{NoSumSMA, SMA};
use chrono::prelude::*;
use crate::fetch::FetchResult;
use std::ops::Add;
use std::time::Duration;

// We actually wait 3xavg fetch duration between fetches
const MIN_FETCH_DURATION: Duration = Duration::from_millis(150);

pub struct Politeness {
    until: DateTime<Utc>,
    duration_avg: NoSumSMA<Duration, u32, 10>,
}

impl Default for Politeness {
    fn default() -> Self {
        Politeness {
            until: Utc::now(),
            duration_avg: NoSumSMA::<Duration, u32, 10>::from_zero(Duration::ZERO),
        }
    }
}

impl Politeness {
    pub fn update(&mut self, fr: &FetchResult) {
        match fr.status.as_u16() {
            200 => {
                self.duration_avg
                    .add_sample(std::cmp::max(fr.duration, MIN_FETCH_DURATION));
                self.until = fr.start.add(self.duration_avg.get_average() * 3);
            }
            429 => todo!(),
            _ => (), // todo
        }
    }

    pub fn wait(&self) {
        crate::clock::wait(self.until);
    }
}
