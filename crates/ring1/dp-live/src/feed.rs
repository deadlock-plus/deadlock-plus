use std::panic::{catch_unwind, AssertUnwindSafe};

use deadlock_reader::supervise::{Attached, ReaderSupervisor, DEFAULT_RETRY_INTERVAL};

use crate::{from_snapshot, LiveFacts};

/// Polls the running game for live facts. Read-only; attaches and reattaches on its own.
pub struct LiveFeed {
    supervisor: ReaderSupervisor,
}

impl Default for LiveFeed {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveFeed {
    pub fn new() -> Self {
        LiveFeed { supervisor: ReaderSupervisor::new(DEFAULT_RETRY_INTERVAL) }
    }

    /// `None` when the game is not running, not attached yet, not in a lobby or match, or the read
    /// failed. A failed read is tolerated by the supervisor; only a run of them drops the reader.
    pub fn poll(&mut self) -> Option<LiveFacts> {
        let reader = match self.supervisor.acquire() {
            Attached::Fresh(r) | Attached::Held(r) => r,
            Attached::Failed(reason) => {
                log::warn!("live feed could not attach to the game: {reason}");
                return None;
            }
            Attached::Absent | Attached::Waiting => return None,
        };
        // The reader walks foreign process memory; a bug in that walk must not take the app down.
        match catch_unwind(AssertUnwindSafe(|| reader.live_snapshot())) {
            Ok(Ok(snapshot)) => {
                self.supervisor.succeeded();
                snapshot.as_ref().map(from_snapshot)
            }
            Ok(Err(e)) => {
                log::debug!("live feed read failed: {e}");
                self.supervisor.failed();
                None
            }
            Err(_) => {
                log::warn!("live feed reader panicked");
                self.supervisor.detach();
                None
            }
        }
    }
}
