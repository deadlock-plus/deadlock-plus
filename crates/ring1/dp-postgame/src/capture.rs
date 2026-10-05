use std::time::Duration;

use deadlock_events::{Engine, Event, Notification, PostGameEvent, PostGameSource};
use deadlock_reader::supervise::{Attached, ReaderSupervisor, DEFAULT_RETRY_INTERVAL};
use deadlock_reader::Reader;

use crate::{to_match, Done, PostGameMatch, Stop, Worker};

const RECV_TIMEOUT: Duration = Duration::from_secs(1);

/// An `Updated` read replaces the earlier `Captured` copy, so both map the same way.
pub fn from_event(event: &PostGameEvent, account_id: u32) -> Option<PostGameMatch> {
    match event {
        PostGameEvent::Captured { metadata, .. } | PostGameEvent::Updated { metadata, .. } => {
            to_match(metadata, account_id)
        }
        _ => None,
    }
}

/// Reads each finished match from the game's memory on its own thread. Dropping or stopping it only
/// signals the thread; it exits on its own within about a second. `on_tick` runs on that thread about once a
/// second with the attached reader, or `None` while the game cannot be reached, so other readers share the one
/// attach.
pub struct Capture {
    worker: Worker,
}

impl Capture {
    /// `after` is the previous capture's `Done`, so the new thread never overlaps the old one.
    pub fn start(
        account_id: u32,
        after: Option<Done>,
        on_match: impl Fn(PostGameMatch) + Send + 'static,
        mut on_tick: impl FnMut(Option<&Reader>) + Send + 'static,
    ) -> std::io::Result<Self> {
        let worker =
            Worker::spawn("postgame-capture", after, move |stop| run(account_id, stop, &on_match, &mut on_tick))?;
        Ok(Capture { worker })
    }

    pub fn stop(self) -> Done {
        self.worker.stop()
    }
}

fn run(account_id: u32, shutdown: &Stop, on_match: &dyn Fn(PostGameMatch), on_tick: &mut dyn FnMut(Option<&Reader>)) {
    let mut supervisor = ReaderSupervisor::new(DEFAULT_RETRY_INTERVAL);
    while !shutdown.is_stopped() {
        let reader = match supervisor.acquire() {
            Attached::Fresh(r) | Attached::Held(r) => r,
            Attached::Failed(reason) => {
                log::warn!("post-game capture could not attach to the game: {reason}");
                on_tick(None);
                shutdown.sleep(DEFAULT_RETRY_INTERVAL);
                continue;
            }
            Attached::Absent | Attached::Waiting => {
                on_tick(None);
                shutdown.sleep(DEFAULT_RETRY_INTERVAL);
                continue;
            }
        };
        let started = Engine::new().with(PostGameSource::new(reader.clone(), account_id)).start();
        let (mut engine, rx) = match started {
            Ok(pair) => pair,
            Err(e) => {
                log::warn!("post-game capture could not start: {e}");
                shutdown.sleep(DEFAULT_RETRY_INTERVAL);
                continue;
            }
        };
        while !shutdown.is_stopped() && engine.is_running() {
            on_tick(Some(&reader));
            match rx.recv_timeout(RECV_TIMEOUT) {
                Ok(Notification::Event { event: Event::PostGame(event), .. }) => match &event {
                    PostGameEvent::Missed { match_id, attempts } => {
                        log::info!("post-game metadata for match {match_id} was not resident after {attempts} reads");
                    }
                    _ => {
                        if let Some(m) = from_event(&event, account_id) {
                            on_match(m);
                        }
                    }
                },
                Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        engine.stop();
        supervisor.detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use valveprotos::deadlock::c_msg_match_meta_data_contents::{MatchInfo, Players};
    use valveprotos::deadlock::CMsgMatchMetaDataContents;

    const ME: u32 = 7;

    fn meta(kills: u32) -> Box<CMsgMatchMetaDataContents> {
        Box::new(CMsgMatchMetaDataContents {
            match_info: Some(MatchInfo {
                match_id: Some(9),
                start_time: Some(1),
                players: vec![Players {
                    account_id: Some(ME),
                    hero_id: Some(1),
                    team: Some(0),
                    kills: Some(kills),
                    ..Default::default()
                }],
                ..Default::default()
            }),
        })
    }

    #[test]
    fn captured_and_updated_map_and_missed_does_not() {
        let captured = PostGameEvent::Captured { match_id: 9, metadata: meta(3) };
        let updated = PostGameEvent::Updated { match_id: 9, metadata: meta(4) };
        let missed = PostGameEvent::Missed { match_id: 9, attempts: 5 };
        assert_eq!(from_event(&captured, ME).unwrap().kills, 3);
        assert_eq!(from_event(&updated, ME).unwrap().kills, 4);
        assert_eq!(from_event(&missed, ME), None);
    }
}
