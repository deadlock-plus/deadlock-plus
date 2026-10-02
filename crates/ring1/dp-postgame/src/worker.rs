use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Cooperative stop flag whose `sleep` wakes as soon as `stop` is called.
#[derive(Default)]
pub struct Stop {
    stopped: Mutex<bool>,
    changed: Condvar,
}

impl Stop {
    pub fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(|e| e.into_inner()) = true;
        self.changed.notify_all();
    }

    pub fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn sleep(&self, duration: Duration) {
        let guard = self.stopped.lock().unwrap_or_else(|e| e.into_inner());
        let _ = self.changed.wait_timeout_while(guard, duration, |stopped| !*stopped);
    }
}

/// Set once the worker thread has fully exited, including after a panic.
#[derive(Clone, Default)]
pub struct Done(Arc<(Mutex<bool>, Condvar)>);

impl Done {
    pub fn is_done(&self) -> bool {
        *self.0 .0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns whether the worker finished within `timeout`.
    pub fn wait(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut done = self.0 .0.lock().unwrap_or_else(|e| e.into_inner());
        while !*done {
            let Some(left) = deadline.checked_duration_since(Instant::now()) else { return false };
            done = self.0 .1.wait_timeout(done, left).unwrap_or_else(|e| e.into_inner()).0;
        }
        true
    }

    fn wait_forever(&self) {
        let mut done = self.0 .0.lock().unwrap_or_else(|e| e.into_inner());
        while !*done {
            done = self.0 .1.wait(done).unwrap_or_else(|e| e.into_inner());
        }
    }

    fn finish(&self) {
        *self.0 .0.lock().unwrap_or_else(|e| e.into_inner()) = true;
        self.0 .1.notify_all();
    }
}

struct FinishOnDrop(Done);

impl Drop for FinishOnDrop {
    fn drop(&mut self) {
        self.0.finish();
    }
}

/// A background thread that is signalled to stop without being joined.
pub struct Worker {
    stop: Arc<Stop>,
    done: Done,
}

impl Worker {
    /// With `after`, the thread first waits for that earlier worker to exit, so a restart never runs two
    /// bodies at once. A stop requested during that wait skips `body` but still finishes only afterwards.
    pub fn spawn(name: &str, after: Option<Done>, body: impl FnOnce(&Stop) + Send + 'static) -> io::Result<Self> {
        let stop = Arc::new(Stop::default());
        let done = Done::default();
        let thread_stop = Arc::clone(&stop);
        let guard = FinishOnDrop(done.clone());
        std::thread::Builder::new().name(name.into()).spawn(move || {
            let _guard = guard;
            if let Some(after) = after {
                after.wait_forever();
            }
            if !thread_stop.is_stopped() {
                body(&thread_stop);
            }
        })?;
        Ok(Worker { stop, done })
    }

    /// Signals the thread and returns at once. The returned handle reports when it has exited.
    pub fn stop(&self) -> Done {
        self.stop.stop();
        self.done.clone()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;

    const LONG: Duration = Duration::from_secs(5);

    #[test]
    fn stop_returns_before_the_body_exits() {
        let (release, gate) = mpsc::channel::<()>();
        let (started_tx, started) = mpsc::channel::<()>();
        let worker = Worker::spawn("t", None, move |_| {
            started_tx.send(()).unwrap();
            let _ = gate.recv();
        })
        .unwrap();
        started.recv_timeout(LONG).unwrap();

        let done = worker.stop();
        assert!(!done.is_done());

        release.send(()).unwrap();
        assert!(done.wait(LONG));
    }

    #[test]
    fn stop_wakes_a_sleeping_body() {
        let (started_tx, started) = mpsc::channel::<()>();
        let worker = Worker::spawn("t", None, move |stop| {
            started_tx.send(()).unwrap();
            stop.sleep(Duration::from_secs(60));
        })
        .unwrap();
        started.recv_timeout(LONG).unwrap();

        let began = Instant::now();
        assert!(worker.stop().wait(LONG));
        assert!(began.elapsed() < Duration::from_secs(30));
    }

    #[test]
    fn a_restart_waits_for_the_previous_body_to_exit() {
        let running = Arc::new(AtomicUsize::new(0));
        let overlap = Arc::new(AtomicUsize::new(0));
        let (release, gate) = mpsc::channel::<()>();
        let (started_tx, started) = mpsc::channel::<()>();

        let (r1, o1) = (Arc::clone(&running), Arc::clone(&overlap));
        let first = Worker::spawn("t", None, move |_| {
            r1.fetch_add(1, Ordering::SeqCst);
            started_tx.send(()).unwrap();
            let _ = gate.recv();
            o1.fetch_max(r1.load(Ordering::SeqCst), Ordering::SeqCst);
            r1.fetch_sub(1, Ordering::SeqCst);
        })
        .unwrap();
        started.recv_timeout(LONG).unwrap();
        let first_done = first.stop();

        let (r2, o2) = (Arc::clone(&running), Arc::clone(&overlap));
        let second = Worker::spawn("t", Some(first_done), move |_| {
            r2.fetch_add(1, Ordering::SeqCst);
            o2.fetch_max(r2.load(Ordering::SeqCst), Ordering::SeqCst);
            r2.fetch_sub(1, Ordering::SeqCst);
        })
        .unwrap();

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(running.load(Ordering::SeqCst), 1);

        release.send(()).unwrap();
        let second_done = second.stop();
        assert!(second_done.wait(LONG));
        assert_eq!(overlap.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_worker_stopped_while_waiting_skips_its_body_and_finishes_after_the_earlier_one() {
        let (release, gate) = mpsc::channel::<()>();
        let (started_tx, started) = mpsc::channel::<()>();
        let first = Worker::spawn("t", None, move |_| {
            started_tx.send(()).unwrap();
            let _ = gate.recv();
        })
        .unwrap();
        started.recv_timeout(LONG).unwrap();
        let first_done = first.stop();

        let ran = Arc::new(AtomicUsize::new(0));
        let ran_in = Arc::clone(&ran);
        let second = Worker::spawn("t", Some(first_done), move |_| {
            ran_in.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
        let second_done = second.stop();

        assert!(!second_done.wait(Duration::from_millis(100)));
        release.send(()).unwrap();
        assert!(second_done.wait(LONG));
        assert_eq!(ran.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_panicking_body_still_reports_done() {
        let worker = Worker::spawn("t", None, |_| panic!("boom")).unwrap();
        assert!(worker.done.wait(LONG));
    }
}
