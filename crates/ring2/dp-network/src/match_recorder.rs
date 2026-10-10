use crate::match_store::{EngineStats, MatchRecording, MatchSample, MatchStore, PingSource};
use crate::types::HistoryPoint;

/// Consecutive non-match observations tolerated before an open recording is closed.
const AWAY_GRACE_TICKS: u32 = 5;
/// A recording that starts with the match clock past this was joined mid-match.
const PARTIAL_AFTER_SECS: f32 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    Other,
    Pregame,
    InMatch,
    PostMatch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineReading {
    pub ping_ms: f32,
    pub stats: EngineStats,
}

/// What one live tick saw.
pub struct Observation<'a> {
    pub now_ms: u64,
    pub phase: LifecyclePhase,
    pub match_id: Option<u64>,
    pub match_time_secs: Option<f32>,
    /// The game's own net channel, when readable this tick.
    pub engine: Option<EngineReading>,
    /// Relay ping history points newer than [`MatchRecorder::relay_since`].
    pub relay: &'a [HistoryPoint],
}

struct OpenFile {
    recording: MatchRecording,
    source: PingSource,
    start_ms: u64,
    last_relay_t: u64,
    bound: Option<u64>,
}

struct Session {
    partial: bool,
    entered_ms: u64,
    match_id: Option<u64>,
    away_ticks: u32,
    /// The source or the file failed: nothing more is written, and no other source takes over.
    dead: bool,
    file: Option<OpenFile>,
}

struct Finished {
    session_key: String,
    bound: Option<u64>,
}

/// Turns live ticks into one ping file per match. It reads no memory itself: the owner feeds it observations.
pub struct MatchRecorder {
    store: MatchStore,
    prefix: String,
    counter: u32,
    prev: LifecyclePhase,
    session: Option<Session>,
    last: Option<Finished>,
}

impl MatchRecorder {
    /// `prefix` makes session keys unique per app run (letters, digits, `-`, `_`).
    pub fn new(store: MatchStore, prefix: impl Into<String>) -> Self {
        Self { store, prefix: prefix.into(), counter: 0, prev: LifecyclePhase::Other, session: None, last: None }
    }

    pub fn observe(&mut self, obs: &Observation<'_>) {
        let prev = std::mem::replace(&mut self.prev, obs.phase);
        match obs.phase {
            LifecyclePhase::InMatch => {
                let new_match = self
                    .session
                    .as_ref()
                    .is_some_and(|s| matches!((s.match_id, obs.match_id), (Some(known), Some(now)) if known != now));
                if new_match {
                    self.close_session();
                }
                let session = self.session.get_or_insert_with(|| Session {
                    partial: prev != LifecyclePhase::Pregame
                        || obs.match_time_secs.is_some_and(|t| t > PARTIAL_AFTER_SECS),
                    entered_ms: obs.now_ms,
                    match_id: None,
                    away_ticks: 0,
                    dead: false,
                    file: None,
                });
                session.away_ticks = 0;
                session.match_id = obs.match_id.or(session.match_id);
                self.sample(obs);
                self.bind_live_id();
            }
            LifecyclePhase::PostMatch => {
                if let Some(session) = &mut self.session {
                    session.match_id = obs.match_id.or(session.match_id);
                }
                self.bind_live_id();
                self.close_session();
            }
            LifecyclePhase::Pregame | LifecyclePhase::Other => {
                let expired = self.session.as_mut().is_some_and(|s| {
                    s.away_ticks += 1;
                    s.away_ticks >= AWAY_GRACE_TICKS
                });
                if expired {
                    self.close_session();
                }
            }
        }
    }

    fn sample(&mut self, obs: &Observation<'_>) {
        let Some(session) = self.session.as_mut().filter(|s| !s.dead) else { return };
        let Some(file) = session.file.as_mut() else {
            if let Some(reading) = obs.engine {
                self.open(PingSource::Engine, obs.now_ms, &[engine_sample(reading, 0)], obs.now_ms);
                return;
            }
            let entered = session.entered_ms;
            let fresh: Vec<&HistoryPoint> = obs.relay.iter().filter(|p| p.t >= entered).collect();
            if let Some(first) = fresh.first() {
                let start = first.t;
                let samples: Vec<MatchSample> = fresh.iter().map(|p| icmp_sample(p, start)).collect();
                let last_t = fresh.last().map_or(start, |p| p.t);
                self.open(PingSource::Icmp, start, &samples, last_t);
            }
            return;
        };
        let failure = match file.source {
            PingSource::Engine => match obs.engine {
                Some(reading) => {
                    let offset = obs.now_ms.saturating_sub(file.start_ms);
                    file.recording.append(&engine_sample(reading, offset)).err().map(|e| e.to_string())
                }
                None => Some("engine ping no longer readable".to_owned()),
            },
            PingSource::Icmp => {
                let mut failure = None;
                let after = file.last_relay_t;
                for p in obs.relay.iter().filter(|p| p.t > after) {
                    if let Err(e) = file.recording.append(&icmp_sample(p, file.start_ms)) {
                        failure = Some(e.to_string());
                        break;
                    }
                    file.last_relay_t = p.t;
                }
                failure
            }
        };
        if let Some(reason) = failure {
            log::warn!("match ping recording stops: {reason}");
            session.dead = true;
        }
    }

    fn open(&mut self, source: PingSource, start_ms: u64, samples: &[MatchSample], last_relay_t: u64) {
        let Some(session) = self.session.as_mut() else { return };
        self.counter += 1;
        let key = format!("{}-{}", self.prefix, self.counter);
        let opened = self.store.start(&key, source, session.partial, start_ms).and_then(|mut recording| {
            for sample in samples {
                recording.append(sample)?;
            }
            Ok(recording)
        });
        match opened {
            Ok(recording) => {
                log::info!("match ping recording started ({source:?}, partial: {})", session.partial);
                session.file = Some(OpenFile { recording, source, start_ms, last_relay_t, bound: None });
            }
            Err(e) => {
                log::warn!("match ping recording could not start: {e}");
                session.dead = true;
            }
        }
    }

    fn bind_live_id(&mut self) {
        let Some(session) = self.session.as_mut() else { return };
        let (Some(id), Some(file)) = (session.match_id, session.file.as_mut()) else { return };
        if file.bound == Some(id) {
            return;
        }
        file.bound = Some(id);
        match self.store.bind_match_id(file.recording.session_key(), id) {
            Ok(true) => {}
            Ok(false) => log::warn!("match ping file for match {id} is missing"),
            Err(e) => log::warn!("could not bind match {id} to its ping file: {e}"),
        }
    }

    fn close_session(&mut self) {
        let Some(session) = self.session.take() else { return };
        let Some(file) = session.file else { return };
        let session_key = file.recording.session_key().to_owned();
        if let Err(e) = file.recording.finish() {
            log::warn!("could not flush match ping file {session_key}: {e}");
        }
        self.last = Some(Finished { session_key, bound: file.bound });
    }

    /// Points to pass in [`Observation::relay`]: only those with `t` above this.
    pub fn relay_since(&self, now_ms: u64) -> u64 {
        let open = self.session.as_ref().filter(|s| !s.dead);
        match open.and_then(|s| s.file.as_ref()) {
            Some(file) => file.last_relay_t,
            None => open.map_or(now_ms, |s| s.entered_ms).saturating_sub(1),
        }
    }

    /// Whether an engine reading can still change anything.
    pub fn needs_engine(&self) -> bool {
        match &self.session {
            Some(s) if s.dead => false,
            Some(Session { file: Some(file), .. }) => file.source == PingSource::Engine,
            _ => true,
        }
    }

    /// Whether relay history points can still change anything.
    pub fn needs_relay(&self) -> bool {
        match &self.session {
            Some(s) if s.dead => false,
            Some(Session { file: Some(file), .. }) => file.source == PingSource::Icmp,
            _ => true,
        }
    }

    /// The post-game step learned `match_id`. Binds the match that just ended; a match still being recorded is
    /// never the target, since post-game data only exists once a match is over.
    pub fn bind_match_id(&mut self, match_id: u64) {
        if self.session.as_ref().is_some_and(|s| s.file.is_some()) {
            return;
        }
        let Some(last) = self.last.as_mut() else { return };
        if last.bound.is_some_and(|bound| bound != match_id) {
            return;
        }
        last.bound = Some(match_id);
        match self.store.bind_match_id(&last.session_key, match_id) {
            Ok(true) => {}
            Ok(false) => log::warn!("match ping file for match {match_id} is missing"),
            Err(e) => log::warn!("could not bind match {match_id} to its ping file: {e}"),
        }
    }

    /// Session key of the open recording, else of the last finished one.
    pub fn session_key(&self) -> Option<&str> {
        match self.session.as_ref().and_then(|s| s.file.as_ref()) {
            Some(file) => Some(file.recording.session_key()),
            None => self.last.as_ref().map(|l| l.session_key.as_str()),
        }
    }
}

fn offset_u32(ms: u64) -> u32 {
    u32::try_from(ms).unwrap_or(u32::MAX)
}

fn engine_sample(reading: EngineReading, offset_ms: u64) -> MatchSample {
    MatchSample { offset_ms: offset_u32(offset_ms), ping_ms: Some(reading.ping_ms), engine: Some(reading.stats) }
}

fn icmp_sample(point: &HistoryPoint, start_ms: u64) -> MatchSample {
    MatchSample { offset_ms: offset_u32(point.t.saturating_sub(start_ms)), ping_ms: point.raw, engine: None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-recorder-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn recorder(name: &str) -> (MatchRecorder, MatchStore, PathBuf) {
        let dir = temp_dir(name);
        (MatchRecorder::new(MatchStore::new(&dir), "run1"), MatchStore::new(&dir), dir)
    }

    fn reading(ping: f32) -> EngineReading {
        EngineReading { ping_ms: ping, stats: EngineStats { loss_down: 0.0, loss_up: 0.0, jitter_ms: 1.0 } }
    }

    fn obs<'a>(now_ms: u64, phase: LifecyclePhase) -> Observation<'a> {
        Observation { now_ms, phase, match_id: None, match_time_secs: Some(0.0), engine: None, relay: &[] }
    }

    fn point(t: u64, raw: Option<f32>) -> HistoryPoint {
        HistoryPoint { t, raw }
    }

    fn files(dir: &PathBuf) -> usize {
        std::fs::read_dir(dir)
            .map(|d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "dpmp")).count())
            .unwrap_or(0)
    }

    use LifecyclePhase::{InMatch, Other, PostMatch, Pregame};

    #[test]
    fn an_engine_recording_starts_in_match_and_stores_each_tick() {
        let (mut r, store, _) = recorder("engine");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        r.observe(&Observation { engine: Some(reading(45.0)), ..obs(3_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();
        r.observe(&obs(4_000, PostMatch));

        let got = store.read_session(&key).unwrap().unwrap();
        assert_eq!(got.header.source, PingSource::Engine);
        assert_eq!(got.header.start_ms, 2_000);
        assert!(!got.header.partial);
        let pings: Vec<_> = got.points.iter().map(|p| (p.offset_ms, p.ping_ms)).collect();
        assert_eq!(pings, vec![(0, Some(40.0)), (1_000, Some(45.0))]);
        assert!(got.points[0].engine.is_some());
    }

    #[test]
    fn without_an_engine_reading_the_relay_ping_is_recorded_from_its_own_timestamps() {
        let (mut r, store, _) = recorder("icmp");
        r.observe(&obs(1_000, Pregame));
        r.observe(&obs(2_000, InMatch));
        let relay = [point(1_500, Some(99.0)), point(2_400, Some(30.0)), point(3_400, None)];
        r.observe(&Observation { relay: &relay, ..obs(3_500, InMatch) });
        let key = r.session_key().unwrap().to_owned();

        let got = store.read_session(&key).unwrap().unwrap();
        assert_eq!(got.header.source, PingSource::Icmp);
        assert_eq!(got.header.start_ms, 2_400, "points from before the match are not part of it");
        let pings: Vec<_> = got.points.iter().map(|p| (p.offset_ms, p.ping_ms)).collect();
        assert_eq!(pings, vec![(0, Some(30.0)), (1_000, None)]);
    }

    #[test]
    fn relay_points_already_stored_are_not_written_twice() {
        let (mut r, store, _) = recorder("icmp-dup");
        r.observe(&obs(1_000, Pregame));
        let relay = [point(2_100, Some(30.0))];
        r.observe(&Observation { relay: &relay, ..obs(2_000, InMatch) });
        r.observe(&Observation { relay: &relay, ..obs(3_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();
        assert_eq!(store.read_session(&key).unwrap().unwrap().points.len(), 1);
    }

    #[test]
    fn joining_mid_match_is_partial() {
        let (mut r, store, _) = recorder("partial");
        r.observe(&obs(1_000, Other));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();
        assert!(store.read_session(&key).unwrap().unwrap().header.partial);
    }

    #[test]
    fn a_late_match_clock_after_pregame_is_partial() {
        let (mut r, store, _) = recorder("late-clock");
        r.observe(&obs(1_000, Pregame));
        let late = Observation { engine: Some(reading(40.0)), match_time_secs: Some(600.0), ..obs(2_000, InMatch) };
        r.observe(&late);
        let key = r.session_key().unwrap().to_owned();
        assert!(store.read_session(&key).unwrap().unwrap().header.partial);
    }

    #[test]
    fn the_live_match_id_binds_as_soon_as_it_is_known() {
        let (mut r, store, _) = recorder("bind-live");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        assert!(store.read(77).unwrap().is_none());
        r.observe(&Observation { engine: Some(reading(41.0)), match_id: Some(77), ..obs(3_000, InMatch) });
        assert_eq!(store.read(77).unwrap().unwrap().points.len(), 2);
    }

    #[test]
    fn the_post_game_id_binds_the_match_that_just_ended() {
        let (mut r, store, _) = recorder("bind-post");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        r.observe(&obs(3_000, PostMatch));
        r.bind_match_id(500);
        assert_eq!(store.read(500).unwrap().unwrap().points.len(), 1);
    }

    #[test]
    fn a_post_game_id_does_not_bind_a_match_still_being_recorded() {
        let (mut r, store, _) = recorder("bind-open");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        r.bind_match_id(500);
        assert!(store.read(500).unwrap().is_none());
    }

    #[test]
    fn a_post_game_id_does_not_replace_a_different_live_id() {
        let (mut r, store, _) = recorder("bind-conflict");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), match_id: Some(7), ..obs(2_000, InMatch) });
        r.observe(&obs(3_000, PostMatch));
        r.bind_match_id(8);
        assert!(store.read(7).unwrap().is_some());
        assert!(store.read(8).unwrap().is_none());
    }

    #[test]
    fn binding_again_with_the_same_id_keeps_the_file() {
        let (mut r, store, _) = recorder("bind-twice");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), match_id: Some(7), ..obs(2_000, InMatch) });
        r.observe(&obs(3_000, PostMatch));
        r.bind_match_id(7);
        assert_eq!(store.read(7).unwrap().unwrap().points.len(), 1);
    }

    #[test]
    fn an_engine_recording_stops_when_the_engine_goes_away_and_never_takes_the_relay() {
        let (mut r, store, _) = recorder("no-mix-engine");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        let relay = [point(3_100, Some(30.0))];
        r.observe(&Observation { relay: &relay, ..obs(3_000, InMatch) });
        r.observe(&Observation { engine: Some(reading(41.0)), ..obs(4_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();

        let got = store.read_session(&key).unwrap().unwrap();
        assert_eq!(got.header.source, PingSource::Engine);
        assert_eq!(got.points.len(), 1);
        assert!(!r.needs_engine());
        assert!(!r.needs_relay());
    }

    #[test]
    fn a_relay_recording_ignores_a_later_engine_reading() {
        let (mut r, store, _) = recorder("no-mix-icmp");
        r.observe(&obs(1_000, Pregame));
        let relay = [point(2_100, Some(30.0))];
        r.observe(&Observation { relay: &relay, ..obs(2_000, InMatch) });
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(3_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();

        let got = store.read_session(&key).unwrap().unwrap();
        assert_eq!(got.header.source, PingSource::Icmp);
        assert_eq!(got.points.len(), 1);
        assert!(!r.needs_engine());
        assert!(r.needs_relay());
    }

    #[test]
    fn nothing_is_recorded_outside_a_match() {
        let (mut r, _, dir) = recorder("outside");
        let relay = [point(1_100, Some(30.0))];
        for (i, phase) in [Other, Pregame, Other, PostMatch].into_iter().enumerate() {
            let now = 1_000 * (i as u64 + 1);
            r.observe(&Observation { engine: Some(reading(40.0)), relay: &relay, ..obs(now, phase) });
        }
        assert_eq!(files(&dir), 0);
        assert!(r.session_key().is_none());
    }

    #[test]
    fn a_short_gap_does_not_split_a_match_but_a_long_one_closes_it() {
        let (mut r, store, dir) = recorder("grace");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(2_000, InMatch) });
        r.observe(&obs(3_000, Other));
        r.observe(&Observation { engine: Some(reading(42.0)), ..obs(4_000, InMatch) });
        assert_eq!(files(&dir), 1);
        let key = r.session_key().unwrap().to_owned();
        assert_eq!(store.read_session(&key).unwrap().unwrap().points.len(), 2);

        for i in 0..AWAY_GRACE_TICKS {
            r.observe(&obs(5_000 + u64::from(i) * 1_000, Other));
        }
        r.observe(&Observation { engine: Some(reading(43.0)), ..obs(20_000, InMatch) });
        assert_eq!(files(&dir), 2);
        assert_ne!(r.session_key().unwrap(), key);
    }

    #[test]
    fn a_new_match_id_starts_a_new_file() {
        let (mut r, _, dir) = recorder("new-id");
        r.observe(&obs(1_000, Pregame));
        r.observe(&Observation { engine: Some(reading(40.0)), match_id: Some(1), ..obs(2_000, InMatch) });
        r.observe(&Observation { engine: Some(reading(40.0)), match_id: Some(2), ..obs(3_000, InMatch) });
        assert_eq!(files(&dir), 2);
    }

    #[test]
    fn a_waiting_recording_starts_on_the_first_sample() {
        let (mut r, store, _) = recorder("waiting");
        r.observe(&obs(1_000, Pregame));
        r.observe(&obs(2_000, InMatch));
        assert!(r.session_key().is_none());
        r.observe(&Observation { engine: Some(reading(40.0)), ..obs(3_000, InMatch) });
        let key = r.session_key().unwrap().to_owned();
        let got = store.read_session(&key).unwrap().unwrap();
        assert_eq!(got.header.start_ms, 3_000);
        assert!(!got.header.partial);
    }
}
