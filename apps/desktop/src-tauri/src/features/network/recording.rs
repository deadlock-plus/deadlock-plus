use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use dp_live::{EnginePing, LiveFacts};
use dp_network::{EngineReading, EngineStats, LifecyclePhase, MatchRecorder, MatchStore, NetworkMonitor, Observation};
use dp_sync::LockExt;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::features::live::state::LivePhase;

pub use dp_storage::MATCH_PING_DIR;

/// Folder of per-match ping files under the app data directory.
pub fn store_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join(MATCH_PING_DIR))
}

fn lifecycle(phase: LivePhase) -> LifecyclePhase {
    match phase {
        LivePhase::Pregame => LifecyclePhase::Pregame,
        LivePhase::InMatch => LifecyclePhase::InMatch,
        LivePhase::PostMatch => LifecyclePhase::PostMatch,
        LivePhase::GameClosed | LivePhase::Menus | LivePhase::Queuing => LifecyclePhase::Other,
    }
}

fn reading(ping: EnginePing) -> EngineReading {
    EngineReading {
        ping_ms: ping.ping_ms,
        stats: EngineStats { loss_down: ping.loss_down, loss_up: ping.loss_up, jitter_ms: ping.jitter_ms },
    }
}

/// The game's own measurement of the user's connection to the match server, in display units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EnginePingView {
    pub ping_ms: f32,
    pub loss_down_pct: f32,
    pub loss_up_pct: f32,
    pub jitter_ms: f32,
}

impl From<EnginePing> for EnginePingView {
    fn from(ping: EnginePing) -> Self {
        let pct = |fraction: f32| (fraction * 100.0).clamp(0.0, 100.0);
        Self {
            ping_ms: ping.ping_ms,
            loss_down_pct: pct(ping.loss_down),
            loss_up_pct: pct(ping.loss_up),
            jitter_ms: ping.jitter_ms,
        }
    }
}

/// Latest engine reading, kept only while a match is running so the display never shows a stale value.
#[derive(Default)]
struct LatestEngine {
    view: Mutex<Option<EnginePingView>>,
}

impl LatestEngine {
    /// Reads the engine once when in a match and keeps the result for display. The recorder gets the same reading
    /// only when it asks for the engine source.
    fn observe(
        &self,
        in_match: bool,
        recorder_needs_engine: bool,
        read: impl FnOnce() -> Option<EnginePing>,
    ) -> Option<EngineReading> {
        let ping = in_match.then(read).flatten();
        *self.view.lock_or_recover() = ping.map(EnginePingView::from);
        ping.filter(|_| recorder_needs_engine).map(reading)
    }

    fn get(&self) -> Option<EnginePingView> {
        *self.view.lock_or_recover()
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Records each match's ping to disk, driven by the live report tick.
#[derive(Default)]
pub struct MatchPingService {
    recorder: Mutex<Option<MatchRecorder>>,
    engine: LatestEngine,
}

impl MatchPingService {
    /// Called once per live tick. `engine` is invoked at most once, and only while a match is running.
    pub fn tick(
        &self,
        app: &AppHandle,
        phase: LivePhase,
        facts: Option<&LiveFacts>,
        engine: impl FnOnce() -> Option<EnginePing>,
    ) {
        let mut slot = self.recorder.lock_or_recover();
        if slot.is_none() {
            if let Some(dir) = store_dir(app) {
                *slot = Some(MatchRecorder::new(MatchStore::new(dir), format!("m{}", now_ms())));
            }
        }
        let in_match = phase == LivePhase::InMatch;
        let needs_engine = slot.as_ref().is_some_and(|recorder| recorder.needs_engine());
        let engine = self.engine.observe(in_match, needs_engine, engine);
        let Some(recorder) = slot.as_mut() else { return };
        let now = now_ms();
        let relay = if in_match && recorder.needs_relay() {
            app.state::<NetworkMonitor>().history_after(Some(recorder.relay_since(now)))
        } else {
            Vec::new()
        };
        recorder.observe(&Observation {
            now_ms: now,
            phase: lifecycle(phase),
            match_id: facts.and_then(|f| f.match_id),
            match_time_secs: facts.and_then(|f| f.match_time_secs),
            engine,
            relay: &relay,
        });
    }

    /// The game's own ping reading from the latest in-match tick; `None` outside a match or when unreadable.
    pub fn engine_latest(&self) -> Option<EnginePingView> {
        self.engine.get()
    }

    /// Session key of the file being written, which storage clearing must leave alone.
    pub fn recording_key(&self) -> Option<String> {
        self.recorder.lock_or_recover().as_ref().and_then(|r| r.recording_key().map(str::to_owned))
    }

    /// The post-game step learned the id of the match that just ended.
    pub fn bind_match_id(&self, match_id: u64) {
        if let Some(recorder) = self.recorder.lock_or_recover().as_mut() {
            recorder.bind_match_id(match_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PING: EnginePing = EnginePing { ping_ms: 31.0, loss_down: 0.025, loss_up: 0.5, jitter_ms: 4.5 };

    #[test]
    fn the_view_reports_loss_as_a_percentage() {
        let view = EnginePingView::from(PING);
        assert_eq!(view, EnginePingView { ping_ms: 31.0, loss_down_pct: 2.5, loss_up_pct: 50.0, jitter_ms: 4.5 });
    }

    #[test]
    fn the_view_clamps_loss_to_a_valid_percentage() {
        let view = EnginePingView::from(EnginePing { loss_down: 1.4, loss_up: -0.2, ..PING });
        assert_eq!((view.loss_down_pct, view.loss_up_pct), (100.0, 0.0));
    }

    #[test]
    fn a_reading_in_a_match_is_shown_even_when_the_recorder_does_not_need_it() {
        let latest = LatestEngine::default();
        let for_recorder = latest.observe(true, false, || Some(PING));
        assert_eq!(for_recorder, None);
        assert_eq!(latest.get(), Some(EnginePingView::from(PING)));
    }

    #[test]
    fn the_recorder_gets_the_same_reading_when_it_needs_the_engine() {
        let latest = LatestEngine::default();
        let mut reads = 0;
        let for_recorder = latest.observe(true, true, || {
            reads += 1;
            Some(PING)
        });
        assert_eq!(reads, 1);
        assert_eq!(for_recorder, Some(reading(PING)));
        assert_eq!(latest.get(), Some(EnginePingView::from(PING)));
    }

    #[test]
    fn leaving_the_match_clears_the_reading_without_reading_the_engine() {
        let latest = LatestEngine::default();
        latest.observe(true, true, || Some(PING));
        let for_recorder = latest.observe(false, true, || panic!("engine must not be read outside a match"));
        assert_eq!(for_recorder, None);
        assert_eq!(latest.get(), None);
    }

    #[test]
    fn an_unreadable_engine_clears_the_reading() {
        let latest = LatestEngine::default();
        latest.observe(true, false, || Some(PING));
        let for_recorder = latest.observe(true, true, || None);
        assert_eq!(for_recorder, None);
        assert_eq!(latest.get(), None);
    }

    #[test]
    fn only_match_phases_map_to_match_lifecycle_phases() {
        assert_eq!(lifecycle(LivePhase::InMatch), LifecyclePhase::InMatch);
        assert_eq!(lifecycle(LivePhase::Pregame), LifecyclePhase::Pregame);
        assert_eq!(lifecycle(LivePhase::PostMatch), LifecyclePhase::PostMatch);
        for phase in [LivePhase::GameClosed, LivePhase::Menus, LivePhase::Queuing] {
            assert_eq!(lifecycle(phase), LifecyclePhase::Other, "{phase:?}");
        }
    }

    #[test]
    fn an_engine_reading_keeps_every_field() {
        let got = reading(EnginePing { ping_ms: 30.0, loss_down: 0.1, loss_up: 0.2, jitter_ms: 4.0 });
        assert_eq!(got.ping_ms, 30.0);
        assert_eq!(got.stats, EngineStats { loss_down: 0.1, loss_up: 0.2, jitter_ms: 4.0 });
    }
}
