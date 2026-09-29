use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use super::{Coalescer, GameFlag, GameWatcher, JobsSnapshot, Policy, Registry, Sleeper};
use crate::features::kv::KvStore;

const EVENT: &str = "jobs://changed";
const EVENT_INTERVAL: Duration = Duration::from_millis(250);
const GAME_POLL: Duration = Duration::from_secs(3);

pub const STORE: &str = "background-jobs";
const POLICIES_KEY: &str = "policies";
const DISABLED_KEY: &str = "disabled";
const ALL_ENABLED_KEY: &str = "allEnabled";
const PAUSE_KEY: &str = "pauseInGame";

pub struct JobsState {
    pub registry: Arc<Registry>,
    events: OnceLock<Arc<Coalescer>>,
    watcher: OnceLock<GameWatcher>,
}

impl Default for JobsState {
    fn default() -> Self {
        let sleeper: Sleeper = Arc::new(std::thread::sleep);
        Self { registry: Registry::new(GameFlag::new(false), sleeper), events: OnceLock::new(), watcher: OnceLock::new() }
    }
}

impl JobsState {
    fn notify(&self) {
        if let Some(events) = self.events.get() {
            events.notify();
        }
    }

    fn persist(&self, app: &AppHandle) {
        let Ok(dir) = app.path().app_data_dir() else { return };
        let kv = app.state::<KvStore>();
        let policies = serde_json::to_value(self.registry.policy_overrides()).unwrap_or_default();
        let mut disabled: Vec<String> = self.registry.disabled_jobs().into_iter().collect();
        disabled.sort();
        let saved = kv
            .set(&dir, STORE, POLICIES_KEY, policies)
            .and_then(|()| kv.set(&dir, STORE, DISABLED_KEY, disabled.into()))
            .and_then(|()| kv.set(&dir, STORE, ALL_ENABLED_KEY, self.registry.all_enabled().into()))
            .and_then(|()| kv.set(&dir, STORE, PAUSE_KEY, self.registry.pause_in_game().into()));
        if let Err(e) = saved {
            log::warn!("could not save background job settings: {e}");
        }
    }
}

fn load_saved(app: &AppHandle, registry: &Registry) {
    let Ok(dir) = app.path().app_data_dir() else { return };
    let kv = app.state::<KvStore>();
    let policies = kv
        .get(&dir, STORE, POLICIES_KEY)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value::<HashMap<String, Policy>>(v).ok())
        .unwrap_or_default();
    for (id, policy) in policies {
        registry.set_policy(&id, policy);
    }
    let disabled = kv
        .get(&dir, STORE, DISABLED_KEY)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();
    for id in disabled {
        registry.set_enabled(&id, false);
    }
    if let Some(enabled) = kv.get(&dir, STORE, ALL_ENABLED_KEY).ok().flatten().and_then(|v| v.as_bool()) {
        registry.set_all_enabled(enabled);
    }
    if let Some(enabled) = kv.get(&dir, STORE, PAUSE_KEY).ok().flatten().and_then(|v| v.as_bool()) {
        registry.set_pause_in_game(enabled);
    }
}

/// Loads saved policies, starts the game poller and the change event. Call once from `setup()`,
/// before any job registers.
pub fn start(app: &AppHandle) {
    let state = app.state::<JobsState>();
    state.registry.declare(crate::features::patch_notes::INDEX_JOB);
    state.registry.declare(crate::features::diagnostics::scan_job::JOB);
    #[cfg(windows)]
    state.registry.declare(crate::features::server_picker::SYNC_JOB);
    load_saved(app, &state.registry);

    let emit_app = app.clone();
    let events = Arc::new(Coalescer::spawn(EVENT_INTERVAL, move || {
        let snapshot = emit_app.state::<JobsState>().registry.snapshot();
        let _ = emit_app.emit(EVENT, snapshot);
    }));
    let listener = events.clone();
    state.registry.set_listener(Box::new(move |_| listener.notify()));
    let _ = state.events.set(events.clone());

    let mut last = None;
    let game = state.registry.game_flag();
    let watcher = GameWatcher::spawn(
        game,
        move || {
            let running = crate::features::voice_bans::game_running();
            if last != Some(running) {
                last = Some(running);
                events.notify();
            }
            running
        },
        GAME_POLL,
    );
    let _ = state.watcher.set(watcher);
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn jobs_snapshot(state: tauri::State<'_, JobsState>) -> Result<JobsSnapshot, ()> {
        Ok(state.registry.snapshot())
    }

    #[tauri::command]
    pub async fn cancel_job(state: tauri::State<'_, JobsState>, id: String) -> Result<(), ()> {
        state.registry.cancel(&id);
        Ok(())
    }

    #[tauri::command]
    pub async fn set_job_policy(
        app: AppHandle,
        state: tauri::State<'_, JobsState>,
        id: String,
        policy: Policy,
    ) -> Result<(), ()> {
        state.registry.set_policy(&id, policy);
        state.persist(&app);
        state.notify();
        Ok(())
    }

    #[tauri::command]
    pub async fn set_pause_in_game(
        app: AppHandle,
        state: tauri::State<'_, JobsState>,
        enabled: bool,
    ) -> Result<(), ()> {
        state.registry.set_pause_in_game(enabled);
        state.persist(&app);
        state.notify();
        Ok(())
    }

    #[tauri::command]
    pub async fn set_job_enabled(
        app: AppHandle,
        state: tauri::State<'_, JobsState>,
        id: String,
        enabled: bool,
    ) -> Result<(), ()> {
        state.registry.set_enabled(&id, enabled);
        state.persist(&app);
        state.notify();
        Ok(())
    }

    #[tauri::command]
    pub async fn set_all_jobs_enabled(
        app: AppHandle,
        state: tauri::State<'_, JobsState>,
        enabled: bool,
    ) -> Result<(), ()> {
        state.registry.set_all_enabled(enabled);
        state.persist(&app);
        state.notify();
        Ok(())
    }

    #[tauri::command]
    pub async fn force_run_job(state: tauri::State<'_, JobsState>, id: String) -> Result<(), ()> {
        state.registry.force_run(&id);
        Ok(())
    }
}
