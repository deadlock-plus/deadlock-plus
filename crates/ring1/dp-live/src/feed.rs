use std::panic::{catch_unwind, AssertUnwindSafe};

use std::time::{SystemTime, UNIX_EPOCH};

use deadlock_reader::steam::active_account_id;
use deadlock_reader::supervise::{Attached, ReaderSupervisor, DEFAULT_RETRY_INTERVAL};
use deadlock_reader::Reader;
use deadlock_walker::{Error as WalkerError, GcSession};

use crate::{from_snapshot, party_facts, LiveFacts, PartyFacts};

/// Polls the running game for live facts. Read-only; attaches and reattaches on its own.
pub struct LiveFeed {
    supervisor: ReaderSupervisor,
    reader: LiveReader,
}

impl Default for LiveFeed {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveFeed {
    pub fn new() -> Self {
        LiveFeed { supervisor: ReaderSupervisor::new(DEFAULT_RETRY_INTERVAL), reader: LiveReader::default() }
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
        match self.reader.read(&reader) {
            Ok(facts) => {
                self.supervisor.succeeded();
                facts
            }
            Err(ReadError::Failed) => {
                self.supervisor.failed();
                None
            }
            Err(ReadError::Panicked) => {
                self.supervisor.detach();
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// The read failed; the next one may succeed.
    Failed,
    /// The reader panicked; the caller should drop its attach.
    Panicked,
}

/// Reads live facts through a reader someone else attached, so a process that already follows the game
/// does not attach a second time.
#[derive(Default)]
pub struct LiveReader {
    session: Option<GcSession>,
}

impl LiveReader {
    /// `Ok(None)` means the game is running but no match or hideout is loaded.
    pub fn read(&mut self, reader: &Reader) -> Result<Option<LiveFacts>, ReadError> {
        // The reader walks foreign process memory; a bug in that walk must not take the app down.
        match catch_unwind(AssertUnwindSafe(|| reader.live_snapshot())) {
            Ok(Ok(snapshot)) => Ok(snapshot.as_ref().map(from_snapshot).map(|mut facts| {
                facts.party = self.read_party(reader);
                facts
            })),
            Ok(Err(e)) => {
                log::debug!("live feed read failed: {e}");
                Err(ReadError::Failed)
            }
            Err(_) => {
                log::warn!("live feed reader panicked");
                self.session = None;
                Err(ReadError::Panicked)
            }
        }
    }

    /// The local party, or `None` when it cannot be read. The session needs the client module and the signed-in Steam
    /// account, so it is built on first use and rebuilt after the game restarts.
    fn read_party(&mut self, reader: &Reader) -> Option<PartyFacts> {
        let read = catch_unwind(AssertUnwindSafe(|| -> Result<Option<PartyFacts>, WalkerError> {
            let mem = reader.memory();
            if self.session.is_none() {
                let Ok(Some(account)) = active_account_id() else { return Ok(None) };
                self.session = Some(GcSession::new(mem, account)?);
            }
            let session = self.session.as_mut().expect("session was just built");
            let party = session.party(mem)?;
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
            Ok(Some(match party {
                None => party_facts(1, None, now, None, None),
                Some(p) => {
                    let raw = |v: Option<i32>| v.and_then(|v| u32::try_from(v).ok());
                    party_facts(p.members.len(), p.match_making_start_time, now, raw(p.match_mode), raw(p.game_mode))
                }
            }))
        }));
        match read {
            Ok(Ok(facts)) => facts,
            Ok(Err(e)) => {
                log::debug!("party read failed: {e}");
                if matches!(e, WalkerError::WrongProcess) {
                    self.session = None;
                }
                None
            }
            Err(_) => {
                log::warn!("party reader panicked");
                self.session = None;
                None
            }
        }
    }
}
