use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Compare RFC 9309, section "2.3.1. Access Results"
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
            _ => self.clone(),
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
    pub fn new() -> Self {
        Self {
            handle: Arc::new(Mutex::new(HashMap::new())),
            last_time_shrinked: SystemTime::now(),
        }
    }

    pub fn get(&self, authority: &str) -> Option<Rc<Entry<T>>> {
        self.handle.lock().unwrap().get(authority).map(Rc::clone)
    }

    pub fn insert(&mut self, authority: &str, ar: AccessResult<T>, now: SystemTime) {
        let mut map = self.handle.lock().unwrap();
        let cachesize = map.len();

        if cachesize > 100
            || (cachesize > 10 && super::elapsed(&self.last_time_shrinked, super::TWO_DAYS))
        {
            self.last_time_shrinked = now;
            let mut delete_older = super::HALF_DAY;
            loop {
                map.retain(|_, v| !super::elapsed(&v.updated, delete_older));
                if map.len() < cachesize {
                    break;
                }
                delete_older /= 2;
            }
        }
        let entry = Rc::new(Entry { ar, updated: now });
        map.insert(authority.to_string(), Rc::clone(&entry));
    }
}
