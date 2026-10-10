use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use dp_live::{EnginePing, LiveFacts};
use dp_network::{EngineReading, EngineStats, LifecyclePhase, MatchRecorder, MatchStore, NetworkMonitor, Observation};
use dp_sync::LockExt;
use tauri::{AppHandle, Manager};

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

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Records each match's ping to disk, driven by the live report tick.
#[derive(Default)]
pub struct MatchPingService {
    recorder: Mutex<Option<MatchRecorder>>,
}

impl MatchPingService {
    /// Called once per live tick. `engine` is only invoked while a match is running and the engine source is still
    /// in play.
    pub fn tick(
        &self,
        app: &AppHandle,
        phase: LivePhase,
        facts: Option<&LiveFacts>,
        engine: impl FnOnce() -> Option<EnginePing>,
    ) {
        let mut slot = self.recorder.lock_or_recover();
        if slot.is_none() {
            let Some(dir) = store_dir(app) else { return };
            *slot = Some(MatchRecorder::new(MatchStore::new(dir), format!("m{}", now_ms())));
        }
        let Some(recorder) = slot.as_mut() else { return };
        let in_match = phase == LivePhase::InMatch;
        let now = now_ms();
        let engine = (in_match && recorder.needs_engine()).then(engine).flatten().map(reading);
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
