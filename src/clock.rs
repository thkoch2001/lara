use chrono::prelude::*;
use chrono::TimeDelta;

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
