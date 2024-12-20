use std::time::{Duration, SystemTime};

pub const HALF_DAY: u64 = 12 * 60 * 60;
pub const ONE_DAY: u64 = 24 * 60 * 60;
pub const TWO_DAYS: u64 = 2 * ONE_DAY;

pub fn elapsed(since: SystemTime, seconds: u64) -> bool {
    let duration = since.elapsed().unwrap_or(Duration::from_secs(1));
    duration.as_secs() > seconds
}
