use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct SignalHandler {
    graceful: Arc<AtomicBool>,
    interrupt: Arc<AtomicBool>,
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self {
            graceful: Arc::new(AtomicBool::new(false)),
            interrupt: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Clone for SignalHandler {
    fn clone(&self) -> Self {
        Self {
            graceful: Arc::clone(&self.graceful),
            interrupt: Arc::clone(&self.interrupt),
        }
    }
}

impl SignalHandler {
    pub fn register() -> Self {
        let handler = SignalHandler::default();
        let ret_val = handler.clone();

        ctrlc::set_handler(move || {
            if handler.graceful.load(Ordering::Relaxed) {
                info!("interrupting gracefully ...");
                handler.interrupt.store(true, Ordering::Relaxed);
            } else {
                info!("interrupting non-gracefully ...");
                std::process::exit(1);
            }
        })
        .expect("Failed to set ctrlc handler");
        ret_val
    }

    pub fn grace(&self) -> Grace {
        assert!(
            !self.graceful.load(Ordering::Relaxed),
            "Only one instance of Grace may exist at any time!"
        );
        self.graceful.store(true, Ordering::Relaxed);
        Grace {
            signal_handler: self.clone(),
        }
    }
}

pub struct Grace {
    signal_handler: SignalHandler,
}

impl Drop for Grace {
    fn drop(&mut self) {
        self.signal_handler.graceful.store(false, Ordering::Relaxed);
    }
}

impl Grace {
    pub fn is_interrupted(&self) -> bool {
        self.signal_handler.interrupt.load(Ordering::Relaxed)
    }
}
