use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// A stop flag a worker can sleep on. `wait` returns as soon as `stop` is called, so a worker blocks for its
/// whole interval instead of polling the flag.
#[derive(Default)]
pub struct StopSignal {
    flag: AtomicBool,
    gate: Mutex<()>,
    changed: Condvar,
}

impl StopSignal {
    pub fn stop(&self) {
        self.flag.store(true, Ordering::Release);
        // Taking the gate orders this store before any waiter's next check, so the notify cannot be missed.
        drop(self.gate.lock().unwrap_or_else(|e| e.into_inner()));
        self.changed.notify_all();
    }

    pub fn is_stopped(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// For code that takes a plain `&AtomicBool` stop flag.
    pub fn flag(&self) -> &AtomicBool {
        &self.flag
    }

    /// Blocks for up to `timeout`. Returns whether the signal is stopped.
    pub fn wait(&self, timeout: Duration) -> bool {
        let guard = self.gate.lock().unwrap_or_else(|e| e.into_inner());
        let _ = self.changed.wait_timeout_while(guard, timeout, |_| !self.is_stopped());
        self.is_stopped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn wait_times_out_when_not_stopped() {
        let signal = StopSignal::default();
        let start = Instant::now();
        assert!(!signal.wait(Duration::from_millis(80)));
        assert!(start.elapsed() >= Duration::from_millis(70));
    }

    #[test]
    fn stop_wakes_a_long_wait_promptly() {
        let signal = Arc::new(StopSignal::default());
        let waiter = {
            let signal = signal.clone();
            std::thread::spawn(move || {
                let start = Instant::now();
                (signal.wait(Duration::from_secs(30)), start.elapsed())
            })
        };
        std::thread::sleep(Duration::from_millis(50));
        signal.stop();
        let (stopped, elapsed) = waiter.join().unwrap();
        assert!(stopped);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn wait_after_stop_returns_at_once() {
        let signal = StopSignal::default();
        signal.stop();
        let start = Instant::now();
        assert!(signal.wait(Duration::from_secs(30)));
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(signal.flag().load(Ordering::Relaxed));
    }
}
