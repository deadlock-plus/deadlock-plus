pub mod commands;

use std::sync::Mutex;

use dp_kv::KvStore;
use dp_postgame::{for_account, reconcile, StoredMatch};
#[cfg(windows)]
use dp_postgame::{plan, upsert, Plan, PostGameMatch};
use dp_sync::LockExt;
#[cfg(windows)]
use tauri::Emitter;
use tauri::{AppHandle, Manager};

#[cfg(windows)]
use super::live::{derive, LivePhase, LiveService, LiveState};
#[cfg(windows)]
use super::toggle::{toggle, Toggle};

const STORE: &str = "postgame-matches";
const KEY: &str = "matches";
#[cfg(windows)]
const EVENT: &str = "postgame-match";
const MAX_AGE_SECS: u64 = 72 * 60 * 60;
#[cfg(windows)]
const EXIT_WAIT: std::time::Duration = std::time::Duration::from_secs(3);
#[cfg(windows)]
const POLL: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(windows)]
#[derive(Default)]
struct WatcherSlot {
    running: Option<dp_postgame::Worker>,
    /// A stopped watcher whose thread may still be exiting. The next start waits on it.
    winding_down: Option<dp_postgame::Done>,
}

#[derive(Default)]
pub struct PostgameService {
    #[cfg(windows)]
    watcher: Mutex<WatcherSlot>,
    /// Serialises load-modify-save between the capture thread and the commands.
    io: Mutex<()>,
}

impl PostgameService {
    /// Idempotent: a watcher that is already running is left alone.
    #[cfg(windows)]
    pub fn start(&self, app: &AppHandle) {
        let mut slot = self.watcher.lock_or_recover();
        if toggle(true, slot.running.is_some()) != Toggle::Start {
            return;
        }
        let handle = app.clone();
        let after = slot.winding_down.take().filter(|done| !done.is_done());
        match dp_postgame::Worker::spawn("postgame-watch", after, move |stop| watch(&handle, stop)) {
            Ok(watcher) => {
                slot.running = Some(watcher);
                log::info!("post-game capture started");
            }
            Err(e) => log::warn!("post-game capture could not start: {e}"),
        }
    }

    #[cfg(not(windows))]
    pub fn start(&self, _app: &AppHandle) {}

    pub fn stop(&self) {
        #[cfg(windows)]
        {
            let done = {
                let mut slot = self.watcher.lock_or_recover();
                match slot.running.take() {
                    Some(watcher) => Some(watcher.stop()),
                    None => slot.winding_down.take(),
                }
            };
            if let Some(done) = done {
                done.wait(EXIT_WAIT);
            }
        }
    }

    pub fn matches(&self, app: &AppHandle, account_id: u32) -> Vec<StoredMatch> {
        let _io = self.io.lock_or_recover();
        for_account(&load(app), account_id)
    }

    pub fn reconcile(&self, app: &AppHandle, account_id: u32, api_ids: &[u64]) -> usize {
        let _io = self.io.lock_or_recover();
        let mut matches = load(app);
        let removed = reconcile(&mut matches, account_id, api_ids, now_secs(), MAX_AGE_SECS);
        if removed > 0 {
            save(app, &matches);
        }
        removed
    }

    #[cfg(windows)]
    fn record(&self, app: &AppHandle, account_id: u32, game: PostGameMatch) {
        let stored = {
            let _io = self.io.lock_or_recover();
            let mut matches = load(app);
            let stored = upsert(&mut matches, account_id, game, now_secs());
            save(app, &matches);
            stored
        };
        log::info!("post-game match {} captured", stored.game.match_id);
        if let Err(e) = app.emit(EVENT, &stored) {
            log::warn!("could not emit {EVENT}: {e}");
        }
    }
}

#[cfg(windows)]
fn watch(app: &AppHandle, stop: &dp_postgame::Stop) {
    let mut running: Option<(u32, dp_postgame::Capture)> = None;
    while !stop.is_stopped() {
        match plan(running.as_ref().map(|(id, _)| *id), dp_steam::current_steam_id32()) {
            Plan::Keep => {}
            Plan::Wait => {
                log::debug!("post-game capture waiting for a signed-in Steam account");
                app.state::<LiveService>().report(app, LiveState::of(LivePhase::GameClosed));
            }
            Plan::Start(id) | Plan::Restart(id) => {
                let after = running.take().map(|(_, capture)| capture.stop());
                let handle = app.clone();
                let ticker = app.clone();
                let mut live = dp_live::LiveReader::default();
                match dp_postgame::Capture::start(
                    id,
                    after,
                    move |game| handle.state::<PostgameService>().record(&handle, id, game),
                    move |attached| {
                        let state = match attached {
                            None => LiveState::of(LivePhase::GameClosed),
                            Some(reader) => match live.read(reader) {
                                Ok(facts) => derive(facts.as_ref()),
                                Err(_) => return,
                            },
                        };
                        ticker.state::<LiveService>().report(&ticker, state);
                    },
                ) {
                    Ok(capture) => {
                        log::info!("post-game capture following account {id}");
                        running = Some((id, capture));
                    }
                    Err(e) => log::warn!("post-game capture could not start: {e}"),
                }
            }
        }
        stop.sleep(POLL);
    }
    if let Some((_, capture)) = running {
        capture.stop().wait(EXIT_WAIT);
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn load(app: &AppHandle) -> Vec<StoredMatch> {
    let Ok(dir) = app.path().app_data_dir() else { return Vec::new() };
    let value = app.state::<KvStore>().get(&dir, STORE, KEY).ok().flatten();
    value.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

fn save(app: &AppHandle, matches: &[StoredMatch]) {
    let Ok(dir) = app.path().app_data_dir() else { return };
    let Ok(value) = serde_json::to_value(matches) else { return };
    if let Err(e) = app.state::<KvStore>().set(&dir, STORE, KEY, value) {
        log::warn!("could not save post-game matches: {e}");
    }
}
