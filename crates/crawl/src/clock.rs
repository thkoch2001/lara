use chrono::TimeDelta;
use chrono::prelude::*;
use std::time::{Duration, Instant};

/// Logger for function execution time
///
/// Logs time until function end to debug log and warns if function took longer than threshold.
///
/// ```
/// # use crawl::clock::TimeToDrop;
/// let _ttd = TimeToDrop::new("FUNCTION_NAME", std::time::Duration::from_millis(1000));
/// ```
pub struct TimeToDrop {
    id: String,
    start: Instant,
    threshold: Duration,
}

impl TimeToDrop {
    pub fn new(id: &str, threshold: Duration) -> Self {
        Self {
            id: String::from(id),
            start: Instant::now(),
            threshold,
        }
    }
}

impl Drop for TimeToDrop {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if duration > self.threshold {
            warn!(
                "{} took {}ms. Threshold: {}ms",
                self.id,
                duration.as_millis(),
                self.threshold.as_millis()
            );
        } else {
            debug!("{} took {}ms.", self.id, duration.as_millis());
        }
    }
}

pub fn elapsed(since: DateTime<Utc>, delta: TimeDelta) -> bool {
    let elapsed_dt = since + delta;
    elapsed_dt <= Utc::now()
}

/// Waits or returns immediately if until already elapsed.
pub fn wait(until: DateTime<Utc>) {
    let delta = until.signed_duration_since(Utc::now());

    match delta.to_std() {
        Err(_) => debug!("Waiting time already elapsed since {delta}"),
        Ok(duration) => {
            debug!("Waiting for {} ms.", duration.as_millis());
            std::thread::sleep(duration);
        }
    }
}

pub fn format_for_filename(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;
    use chrono::TimeDelta;
    use chrono::Utc;

    #[test]
    fn test_elapsed() {
        let delta = TimeDelta::hours(1);
        let mut since = Utc::now();

        assert!(!elapsed(since, delta));

        since = since.checked_sub_days(Days::new(1)).unwrap();
        assert!(elapsed(since, delta));
    }
}
