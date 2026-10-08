use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Instant;

use deadlock_reader::supervise::{Attached, ReaderSupervisor, DEFAULT_RETRY_INTERVAL};
use deadlock_reader::Reader;
use deadlock_walker::{Error as WalkerError, QueueFlag};

use crate::convert::{hideout_party_size, queue_state};
use crate::link::{Backend, LiveRead, ReadError, Snapshot};
use crate::party_policy::in_match;
use crate::probes::{ProbeFeed, QueueSource, PROBE_INTERVAL};
use crate::queue::{ProbeError, QueuePoller, QueueProbe, QueueState};
use crate::{board_from_snapshot, from_snapshot};

/// The live-facts side of a [`crate::GameLink`]: the supervised attach plus the reader that walks it.
#[derive(Default)]
pub struct SupervisedBackend {
    supervisor: Option<ReaderSupervisor>,
    reader: LiveReader,
}

impl SupervisedBackend {
    fn supervisor(&mut self) -> &mut ReaderSupervisor {
        self.supervisor.get_or_insert_with(|| ReaderSupervisor::new(DEFAULT_RETRY_INTERVAL))
    }
}

impl Backend for SupervisedBackend {
    type Reader = Arc<Reader>;

    fn acquire(&mut self) -> Option<Arc<Reader>> {
        match self.supervisor().acquire() {
            Attached::Fresh(r) => {
                self.reader.start(&r);
                Some(r)
            }
            Attached::Held(r) => Some(r),
            Attached::Failed(reason) => {
                log::warn!("game link could not attach to the game: {reason}");
                None
            }
            Attached::Absent | Attached::Waiting => None,
        }
    }

    fn read(&mut self, reader: &Arc<Reader>) -> Result<Snapshot, ReadError> {
        self.reader.read_full(reader)
    }

    fn succeeded(&mut self) {
        self.supervisor().succeeded();
    }

    fn failed(&mut self) {
        self.supervisor().failed();
    }

    fn detach(&mut self) {
        self.reader.stop();
        self.supervisor().detach();
    }
}

/// Walks the attached game for one tick's facts and scoreboard. The queue-flag poll runs on its own thread, so a tick
/// only reads what that thread last published.
#[derive(Default)]
pub(crate) struct LiveReader {
    probes: ProbeFeed,
}

impl QueueProbe for QueueFlag {
    type Mem = Reader;

    fn poll(&mut self, reader: &Reader) -> Result<Option<QueueState>, ProbeError> {
        let mem = reader.memory();
        let read = QueueFlag::queueing(self, mem).and_then(|queueing| match queueing {
            Some(true) => self.request(mem).map(|request| Some(queue_state(true, request))),
            Some(false) => Ok(Some(queue_state(false, None))),
            None => Ok(None),
        });
        read.map_err(|e| match e {
            WalkerError::WrongProcess => ProbeError::WrongProcess,
            e => {
                log::debug!("queue probe failed: {e}");
                ProbeError::Failed
            }
        })
    }
}

impl LiveReader {
    /// Begins probing a freshly attached game, replacing any earlier probes.
    pub fn start(&mut self, reader: &Arc<Reader>) {
        if let Err(e) =
            self.probes.start(GameQueue { reader: Arc::clone(reader), poller: QueuePoller::default() }, PROBE_INTERVAL)
        {
            log::warn!("could not start the game probes: {e}");
        }
    }

    pub fn stop(&mut self) {
        self.probes.stop();
    }

    /// `Ok(None)` means the game is running but no match or hideout is loaded.
    pub fn read_full(&mut self, reader: &Reader) -> Result<Option<LiveRead>, ReadError> {
        // The reader walks foreign process memory; a bug in that walk must not take the app down.
        match catch_unwind(AssertUnwindSafe(|| reader.live_snapshot())) {
            Ok(Ok(snapshot)) => Ok(snapshot.as_ref().map(|snap| {
                let mut facts = from_snapshot(snap);
                let board = board_from_snapshot(snap);
                facts.party =
                    self.probes.party(Instant::now(), in_match(&facts), hideout_party_size(facts.context, &board));
                LiveRead { facts, board }
            })),
            Ok(Err(e)) => {
                log::debug!("live feed read failed: {e}");
                Err(ReadError::Failed)
            }
            Err(_) => {
                log::warn!("live feed reader panicked");
                Err(ReadError::Panicked)
            }
        }
    }
}

/// The queue-flag probe, built on first use; the queue thread owns it.
struct GameQueue {
    reader: Arc<Reader>,
    poller: QueuePoller<QueueFlag>,
}

impl QueueSource for GameQueue {
    fn queueing(&mut self) -> Option<QueueState> {
        let reader = Arc::clone(&self.reader);
        self.poller.poll(Instant::now(), &reader, |r| match QueueFlag::new(r.memory()) {
            Ok(ui) => Some(ui),
            Err(e) => {
                log::debug!("queue probe unavailable: {e}");
                None
            }
        })
    }
}
