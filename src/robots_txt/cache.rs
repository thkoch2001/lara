use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Compare RFC 9309, section "2.3.1. Access Results"
#[derive(Debug, PartialEq)]
pub enum AccessResult<T> {
    /// HTTP 400-499 range
    Unavailable,
    /// HTTP 500-599 range
    Unreachable(SystemTime),
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
    pub updated: SystemTime,
}

type Handle<T> = Arc<Mutex<HashMap<String, Rc<Entry<T>>>>>;
pub struct Cache<T> {
    handle: Handle<T>,
    last_time_shrinked: SystemTime,
}

impl<T> Cache<T> {
    pub fn new(now: SystemTime) -> Self {
        Self {
            handle: Arc::new(Mutex::new(HashMap::new())),
            last_time_shrinked: now,
        }
    }

    pub fn get(&self, authority: &str) -> Option<Rc<Entry<T>>> {
        self.handle.lock().unwrap().get(authority).map(Rc::clone)
    }

    pub fn insert(&mut self, authority: &str, ar: AccessResult<T>, now: SystemTime) {
        debug!("Insert for {authority} in robots.txt cache");
        let mut map = self.handle.lock().unwrap();
        let cachesize = map.len();

        if cachesize > 100
            || (cachesize > 10 && super::elapsed(self.last_time_shrinked, super::TWO_DAYS))
        {
            self.last_time_shrinked = now;
            let mut delete_older = super::HALF_DAY;
            loop {
                map.retain(|_, v| !super::elapsed(v.updated, delete_older));
                if map.len() < cachesize {
                    break;
                }
                delete_older /= 2;
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

    use super::{AccessResult as AR, Cache};
    use assertables::*;
    use proptest::prelude::*;
    use std::time::SystemTime;

    #[test]
    fn access_result_clone() {
        let ar: AR<()> = AR::Unavailable;
        assert_eq!(ar, ar.clone());

        let ar: AR<bool> = AR::Ok(std::rc::Rc::new(true));
        assert_eq!(ar, ar.clone());

        let ar: AR<i64> = AR::Unreachable(SystemTime::now());
        assert_eq!(ar, ar.clone());
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            max_shrink_iters: 20,
            cases: 100,
            .. ProptestConfig::default()
        })]
        #[test]
        fn test_insert(inserts in proptest::collection::vec(("\\PC*", any::<u8>()), 50..300)) {
            let mut now = SystemTime::UNIX_EPOCH;
            let mut cache: Cache<()> = Cache::new(now);
            let mut len_before = cache.len();

            assert_eq!(len_before, 0);

            for (authority, delay) in inserts {
                let duration = std::time::Duration::from_secs(3000 * u64::from(delay));
                now = now.checked_add(duration).unwrap();
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
