use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

// TODO: This Cache needs code to expire old entries

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

impl<T> Entry<T> {
    fn create_with_now(ar: AccessResult<T>) -> Self {
        Self {
            ar,
            updated: SystemTime::now(),
        }
    }
}

type Handle<T> = Arc<Mutex<HashMap<String, Rc<Entry<T>>>>>;
pub struct Cache<T> {
    handle: Handle<T>,
}

impl<T> Cache<T> {
    pub fn new() -> Self {
        Self {
            handle: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, authority: &str) -> Option<Rc<Entry<T>>> {
        self.handle.lock().unwrap().get(authority).map(Rc::clone)
    }

    pub fn insert(&self, authority: &str, ar: AccessResult<T>) {
        let entry = Rc::new(Entry::create_with_now(ar));
        self.handle
            .lock()
            .unwrap()
            .insert(authority.to_string(), Rc::clone(&entry));
    }
}
