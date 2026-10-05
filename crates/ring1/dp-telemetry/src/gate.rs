use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Closed until the user's choice is known, so nothing is sent before the notice has been shown.
#[derive(Debug, Clone, Default)]
pub struct Gate(Arc<AtomicBool>);

impl Gate {
    pub fn is_open(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn set(&self, open: bool) {
        self.0.store(open, Ordering::SeqCst);
    }
}
