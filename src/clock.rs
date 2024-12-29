use std::time::{Duration, SystemTime, SystemTimeError};

pub const HALF_DAY: u64 = 12 * 60 * 60;
pub const ONE_DAY: u64 = 24 * 60 * 60;
pub const TWO_DAYS: u64 = 2 * ONE_DAY;

pub fn elapsed(since: SystemTime, seconds: u64) -> bool {
    let duration = since.elapsed().unwrap_or(Duration::from_secs(1));
    duration.as_secs() > seconds
}

/// Waits or returns immediately if until already elapsed.
pub fn wait(until: SystemTime) {
    let now = SystemTime::now();
    match until.duration_since(now) {
        Err(e) => debug!("Waiting time already elapsed since {:?}", e.duration()),
        Ok(d) => {
            debug!("Waiting for {} ms.", d.as_millis());
            std::thread::sleep(d);
        }
    };
}
