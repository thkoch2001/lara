use crawl::clock;

use chrono::prelude::*;
use chrono::TimeDelta as TD;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Compare RFC 9309, section "2.3.1. Access Results"
#[derive(Debug, PartialEq)]
pub enum AccessResult<T> {
    /// HTTP 400-499 range
    Unavailable,
    /// HTTP 500-599 range
    Unreachable(DateTime<Utc>),
    Ok(Rc<T>),
}

impl<T> Clone for AccessResult<T> {
    fn clone(&self) -> AccessResult<T> {
        match self {
            AccessResult::Ok(rc) => AccessResult::Ok(Rc::clone(rc)),
            AccessResult::Unavailable => AccessResult::Unavailable,
            AccessResult::Unreachable(st) => AccessResult::Unreachable(*st),
        }
    }
}

/// Cache entry
pub struct Entry<T> {
    pub ar: AccessResult<T>,
    /// Time of last update for this cache entry
    pub updated: DateTime<Utc>,
}

type Handle<T> = Arc<Mutex<HashMap<String, Rc<Entry<T>>>>>;
pub struct Cache<T> {
    handle: Handle<T>,
    last_time_shrinked: DateTime<Utc>,
}

impl<T> Cache<T> {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            handle: Arc::new(Mutex::new(HashMap::new())),
            last_time_shrinked: now,
        }
    }

    pub fn get(&self, authority: &str) -> Option<Rc<Entry<T>>> {
        self.handle.lock().unwrap().get(authority).map(Rc::clone)
    }

    pub fn insert(&mut self, authority: &str, ar: AccessResult<T>, now: DateTime<Utc>) {
        debug!("Insert for {authority} in robots.txt cache");
        let mut map = self.handle.lock().unwrap();
        let cachesize = map.len();

        if cachesize >= 100
            || (cachesize > 10 && clock::elapsed(self.last_time_shrinked, TD::days(2)))
        {
            self.last_time_shrinked = now;
            let mut delete_older = TD::hours(12);
            loop {
                map.retain(|_, v| !clock::elapsed(v.updated, delete_older));
                if map.len() < cachesize {
                    break;
                }
                delete_older = delete_older / 2;
                #[cfg(test)]
                assert!(delete_older < TD::hours(7))
            }
        }
        let entry = Rc::new(Entry { ar, updated: now });
        map.insert(authority.to_string(), Rc::clone(&entry));
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.handle.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use chrono::Utc;

    use super::{AccessResult as AR, Cache};
    use assertables::*;
    use proptest::prelude::*;

    #[test]
    fn access_result_clone() {
        let ar: AR<()> = AR::Unavailable;
        assert_eq!(ar, ar.clone());

        let ar: AR<bool> = AR::Ok(std::rc::Rc::new(true));
        assert_eq!(ar, ar.clone());

        let ar: AR<i64> = AR::Unreachable(Utc::now());
        assert_eq!(ar, ar.clone());
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            max_shrink_iters: 20,
            cases: 100,
            .. ProptestConfig::default()
        })]
        #[test]
        fn test_insert(inserts in proptest::collection::vec(("\\PC*", 0i64..256), 50..300)) {
            let mut now = Utc::now().checked_sub_months(chrono::Months::new(120)).unwrap();
            let mut cache: Cache<()> = Cache::new(now);
            let mut len_before = cache.len();

            assert_eq!(len_before, 0);

            for (authority, delay) in inserts {
                let delta = TimeDelta::try_seconds(3000 * delay).unwrap();
                now = now.checked_add_signed(delta).unwrap();
                assert!(now <= Utc::now());
                cache.insert(authority.as_str(), AR::Unavailable, now);
                assert_eq!(
                    cache.get(authority.as_str()).unwrap().updated,
                    now
                );
                assert_le!(cache.len(), 100);
                assert_le!(cache.len(), len_before + 1);
                // cache length could have remained constant in case of already existing key
                if cache.len() < len_before {
                    assert_eq!(cache.last_time_shrinked, now);
                }
                len_before = cache.len();
            }
        }
    }
}
