mod activity;
mod hub;
mod live;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dp_discord_ipc::ClientKind;
use dp_game::start_time;
use dp_presence::{map, with_support_button, GameFacts, PresenceLevel};
use dp_sync::LockExt;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::toggle::{toggle, Toggle};
use hub::Hub;
use live::{convert, LiveSource, PlatformFeed};

const APPLICATION_ID: &str = "1467944678002397328";
const POLL: Duration = Duration::from_secs(2);
const SLICE: Duration = Duration::from_millis(100);
/// How long app exit and a restarting worker wait for a worker to clear and disconnect. Discord I/O has no
/// timeout, so a stuck connection must not hold the app open.
const SHUTDOWN_GRACE: Duration = Duration::from_millis(1500);

#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum PresenceLevelSetting {
    #[default]
    Off,
    Basic,
    Detailed,
}

impl From<PresenceLevelSetting> for PresenceLevel {
    fn from(level: PresenceLevelSetting) -> Self {
        match level {
            PresenceLevelSetting::Off => Self::Off,
            PresenceLevelSetting::Basic => Self::Basic,
            PresenceLevelSetting::Detailed => Self::Detailed,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum DiscordClientKind {
    Stable,
    Ptb,
    Canary,
    Other,
}

impl From<DiscordClientKind> for ClientKind {
    fn from(kind: DiscordClientKind) -> Self {
        match kind {
            DiscordClientKind::Stable => Self::Stable,
            DiscordClientKind::Ptb => Self::Ptb,
            DiscordClientKind::Canary => Self::Canary,
            DiscordClientKind::Other => Self::Other,
        }
    }
}

impl From<ClientKind> for DiscordClientKind {
    fn from(kind: ClientKind) -> Self {
        match kind {
            ClientKind::Stable => Self::Stable,
            ClientKind::Ptb => Self::Ptb,
            ClientKind::Canary => Self::Canary,
            ClientKind::Other => Self::Other,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PresenceSettings {
    pub level: PresenceLevelSetting,
    pub clients: Vec<DiscordClientKind>,
    #[serde(default = "default_support_button")]
    pub support_button: bool,
}

fn default_support_button() -> bool {
    true
}

impl Default for PresenceSettings {
    fn default() -> Self {
        Self { level: PresenceLevelSetting::default(), clients: Vec::new(), support_button: true }
    }
}

impl PresenceSettings {
    fn selected(&self) -> HashSet<ClientKind> {
        self.clients.iter().map(|k| ClientKind::from(*k)).collect()
    }
}

#[derive(Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PresenceClient {
    pub kind: DiscordClientKind,
    pub pipe_index: u8,
    pub connected: bool,
}

#[derive(Serialize, Clone, Default, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PresenceStatus {
    pub running: bool,
    pub clients: Vec<PresenceClient>,
}

struct Worker {
    stop: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
}

#[derive(Default)]
struct Slots {
    active: Option<Worker>,
    retired: Option<Arc<AtomicBool>>,
}

#[derive(Default)]
pub struct PresenceService {
    settings: Arc<Mutex<PresenceSettings>>,
    connected: Arc<Mutex<Vec<u8>>>,
    heroes: Arc<Mutex<HashMap<u32, String>>>,
    slots: Mutex<Slots>,
}

struct DoneOnDrop(Arc<AtomicBool>);

impl Drop for DoneOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

fn wait_done(done: &AtomicBool, deadline: Instant) {
    while !done.load(Ordering::Acquire) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
}

impl PresenceService {
    pub fn apply(&self, settings: PresenceSettings) {
        let enabled = settings.level != PresenceLevelSetting::Off;
        *self.settings.lock_or_recover() = settings;

        let mut slots = self.slots.lock_or_recover();
        match toggle(enabled, slots.active.is_some()) {
            Toggle::Keep => {}
            Toggle::Stop => {
                if let Some(worker) = slots.active.take() {
                    worker.stop.store(true, Ordering::Relaxed);
                    slots.retired = Some(worker.done);
                }
                log::info!("discord presence disabled");
            }
            Toggle::Start => {
                log::info!("discord presence enabled");
                let worker = Worker { stop: Arc::default(), done: Arc::default() };
                let (stop, done) = (worker.stop.clone(), worker.done.clone());
                let previous = slots.retired.take();
                slots.active = Some(worker);
                let settings = self.settings.clone();
                let connected = self.connected.clone();
                let heroes = self.heroes.clone();
                std::thread::Builder::new()
                    .name("presence".into())
                    .spawn(move || {
                        let _done = DoneOnDrop(done);
                        if let Some(previous) = previous {
                            wait_done(&previous, Instant::now() + SHUTDOWN_GRACE);
                        }
                        run(&stop, &settings, &connected, &heroes);
                        connected.lock_or_recover().clear();
                    })
                    .expect("spawn thread");
            }
        }
    }

    pub fn set_hero_names(&self, names: HashMap<u32, String>) {
        *self.heroes.lock_or_recover() = names;
    }

    pub fn status(&self) -> PresenceStatus {
        let connected = self.connected.lock_or_recover().clone();
        let clients = dp_discord_ipc::discover()
            .into_iter()
            .map(|p| PresenceClient {
                kind: p.effective_kind().into(),
                pipe_index: p.index,
                connected: connected.contains(&p.index),
            })
            .collect();
        PresenceStatus { running: dp_game::is_running(), clients }
    }

    pub fn stop(&self) {
        let deadline = Instant::now() + SHUTDOWN_GRACE;
        let mut slots = self.slots.lock_or_recover();
        if let Some(worker) = slots.active.take() {
            worker.stop.store(true, Ordering::Relaxed);
            slots.retired = Some(worker.done);
        }
        if let Some(done) = slots.retired.take() {
            wait_done(&done, deadline);
        }
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}

fn sleep_unless_stopped(stop: &AtomicBool, total: Duration) {
    let end = Instant::now() + total;
    while !stop.load(Ordering::Relaxed) && Instant::now() < end {
        std::thread::sleep(SLICE);
    }
}

fn run(
    stop: &AtomicBool,
    settings: &Mutex<PresenceSettings>,
    connected: &Mutex<Vec<u8>>,
    heroes: &Mutex<HashMap<u32, String>>,
) {
    let origin = Instant::now();
    let mut hub = Hub::new();
    let mut source: LiveSource<PlatformFeed> = LiveSource::new();
    while !stop.load(Ordering::Relaxed) {
        let current = settings.lock_or_recover().clone();
        let started = start_time();
        let running = started.is_some();
        let detailed = current.level == PresenceLevelSetting::Detailed;
        let live =
            source.poll(detailed && running, PlatformFeed::default).map(|l| convert(&l, &heroes.lock_or_recover()));
        let facts =
            GameFacts { running, started_at: started.and_then(|t| i64::try_from(t).ok()), now_secs: unix_now(), live };
        let desired = map(current.level.into(), &facts).map(|p| with_support_button(p, current.support_button));
        let targets = if facts.running {
            dp_discord_ipc::select_targets(&current.selected(), &dp_discord_ipc::discover())
        } else {
            Vec::new()
        };
        let now_ms = u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX);
        hub.tick(&targets, &desired, now_ms, |pipe| {
            dp_discord_ipc::connect(pipe, APPLICATION_ID)
                .inspect_err(|e| log::debug!("discord pipe {} unavailable: {e}", pipe.index))
                .ok()
        });
        *connected.lock_or_recover() = hub.connected();
        sleep_unless_stopped(stop, POLL);
    }
    hub.shutdown();
}

pub mod commands {
    use std::collections::HashMap;

    use tauri::State;

    use super::{PresenceService, PresenceSettings, PresenceStatus};

    #[tauri::command]
    pub fn set_presence_settings(settings: PresenceSettings, presence: State<'_, PresenceService>) {
        presence.apply(settings);
    }

    #[tauri::command]
    pub fn set_presence_hero_names(names: HashMap<u32, String>, presence: State<'_, PresenceService>) {
        presence.set_hero_names(names);
    }

    #[tauri::command]
    pub fn presence_status(presence: State<'_, PresenceService>) -> PresenceStatus {
        presence.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_use_camel_case_and_lowercase_names() {
        let s: PresenceSettings = serde_json::from_str(r#"{"level":"basic","clients":["stable","ptb"]}"#).unwrap();
        assert!(s.level == PresenceLevelSetting::Basic);
        assert!(s.clients == [DiscordClientKind::Stable, DiscordClientKind::Ptb]);
    }

    #[test]
    fn support_button_defaults_on_and_reads_camel_case() {
        let old: PresenceSettings = serde_json::from_str(r#"{"level":"basic","clients":[]}"#).unwrap();
        assert!(old.support_button);
        assert!(PresenceSettings::default().support_button);
        let off: PresenceSettings =
            serde_json::from_str(r#"{"level":"basic","clients":[],"supportButton":false}"#).unwrap();
        assert!(!off.support_button);
    }

    #[test]
    fn detailed_level_round_trips_and_maps() {
        let s: PresenceSettings = serde_json::from_str(r#"{"level":"detailed","clients":[]}"#).unwrap();
        assert!(s.level == PresenceLevelSetting::Detailed);
        assert_eq!(serde_json::to_string(&s.level).unwrap(), r#""detailed""#);
        assert_eq!(PresenceLevel::from(s.level), PresenceLevel::Detailed);
    }

    #[test]
    fn hero_names_are_replaced_not_merged() {
        let svc = PresenceService::default();
        svc.set_hero_names(HashMap::from([(1, "A".to_owned())]));
        svc.set_hero_names(HashMap::from([(2, "B".to_owned())]));
        assert_eq!(*svc.heroes.lock_or_recover(), HashMap::from([(2, "B".to_owned())]));
    }

    #[test]
    fn status_serializes_camel_case() {
        let s = PresenceStatus {
            running: true,
            clients: vec![PresenceClient { kind: DiscordClientKind::Canary, pipe_index: 1, connected: false }],
        };
        assert_eq!(
            serde_json::to_string(&s).unwrap(),
            r#"{"running":true,"clients":[{"kind":"canary","pipeIndex":1,"connected":false}]}"#
        );
    }

    #[test]
    fn selected_maps_every_kind() {
        let s = PresenceSettings {
            level: PresenceLevelSetting::Basic,
            clients: vec![
                DiscordClientKind::Stable,
                DiscordClientKind::Ptb,
                DiscordClientKind::Canary,
                DiscordClientKind::Other,
            ],
            support_button: false,
        };
        assert_eq!(s.selected(), ClientKind::all());
    }

    #[test]
    fn off_is_the_default_and_starts_no_worker() {
        let svc = PresenceService::default();
        svc.apply(PresenceSettings::default());
        svc.apply(PresenceSettings::default());
        assert!(svc.slots.lock_or_recover().active.is_none());
    }

    #[test]
    fn repeated_settings_keep_one_worker_and_off_retires_it() {
        let svc = PresenceService::default();
        let on = PresenceSettings {
            level: PresenceLevelSetting::Basic,
            clients: vec![DiscordClientKind::Stable],
            support_button: false,
        };
        svc.apply(on.clone());
        let first = Arc::as_ptr(&svc.slots.lock_or_recover().active.as_ref().unwrap().stop);
        svc.apply(on);
        assert_eq!(first, Arc::as_ptr(&svc.slots.lock_or_recover().active.as_ref().unwrap().stop));
        svc.apply(PresenceSettings::default());
        assert!(svc.slots.lock_or_recover().active.is_none());
        svc.stop();
    }
}
