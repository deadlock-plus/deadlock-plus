use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::time::Instant;
use ts_rs::TS;

use dp_gc::auth::{self, AuthContext};
use dp_gc::client::{GcSession, RecoveredSalts};
use dp_gc::pass::{fetch_loop, AccountState, Host, GC_BACKOFF_SECS};
use dp_gc::picks::fresh_newest_first;
use dp_ingest::salts::Salts;
use dp_kv::KvStore;
use dp_sync::LockExt;

use super::toggle::{toggle, Toggle};

const TO_FETCH_URL: &str = "https://api.deadlock-api.com/v1/matches/to-fetch";
const SALTS_URL: &str = "https://api.deadlock-api.com/v1/matches/salts";
const STORE: &str = "gc-state";
const ACCOUNTS_KEY: &str = "accounts";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(20);
const PASS_INTERVAL: Duration = Duration::from_secs(30 * 60);
const POLL: Duration = Duration::from_secs(1);

#[derive(Serialize, Clone, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GcStatus {
    pub running: bool,
    pub accounts: u32,
    pub delivered: u64,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct GcService {
    stop: Mutex<Option<Arc<AtomicBool>>>,
    status: Arc<Mutex<GcStatus>>,
}

impl GcService {
    pub fn set_enabled(&self, enabled: bool, app: AppHandle, http: reqwest::Client) {
        let mut slot = self.stop.lock_or_recover();
        match toggle(enabled, slot.is_some()) {
            Toggle::Keep => return,
            Toggle::Stop => {
                if let Some(flag) = slot.take() {
                    flag.store(true, Ordering::Relaxed);
                }
                log::info!("gc salt recovery disabled");
                self.status.lock_or_recover().running = false;
                return;
            }
            Toggle::Start => log::info!("gc salt recovery enabled"),
        }

        let flag = Arc::new(AtomicBool::new(false));
        *slot = Some(flag.clone());
        let status = self.status.clone();
        status.lock_or_recover().running = true;
        std::thread::Builder::new()
            .name("gc-salt-recovery".into())
            .spawn(move || {
                run(&flag, &status, &app, &http);
                if !flag.load(Ordering::Relaxed) {
                    status.lock_or_recover().running = false;
                }
            })
            .expect("spawn thread");
    }

    pub fn status(&self) -> GcStatus {
        self.status.lock_or_recover().clone()
    }

    pub fn stop(&self) {
        if let Some(flag) = self.stop.lock_or_recover().take() {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

pub fn to_ingest_salts(salts: RecoveredSalts) -> Salts {
    Salts {
        match_id: salts.match_id,
        cluster_id: salts.cluster_id,
        metadata_salt: salts.metadata_salt,
        replay_salt: salts.replay_salt,
        username: Some(salts.account_id),
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

type States = HashMap<String, AccountState>;

fn load_states(app: &AppHandle) -> States {
    let Ok(dir) = app.path().app_data_dir() else { return States::new() };
    let value = app.state::<KvStore>().get(&dir, STORE, ACCOUNTS_KEY).ok().flatten();
    value.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

fn save_states(app: &AppHandle, states: &States) {
    let Ok(dir) = app.path().app_data_dir() else { return };
    let Ok(value) = serde_json::to_value(states) else { return };
    if let Err(e) = app.state::<KvStore>().set(&dir, STORE, ACCOUNTS_KEY, value) {
        log::warn!("could not save gc state: {e}");
    }
}

struct PassHost<'a> {
    stop: &'a AtomicBool,
    http: &'a reqwest::Client,
    last_request: &'a mut Option<Instant>,
    delivered: u64,
}

impl Host for PassHost<'_> {
    async fn before_request(&mut self) -> bool {
        if let Some(prev) = *self.last_request {
            tokio::time::sleep_until(prev + MIN_REQUEST_INTERVAL).await;
        }
        *self.last_request = Some(Instant::now());
        // Steam routes GC traffic to the game's own pipe while it runs.
        !self.stop.load(Ordering::Relaxed) && !dp_game::is_running()
    }

    async fn deliver(&mut self, salts: RecoveredSalts) -> bool {
        let body = [to_ingest_salts(salts)];
        match self.http.post(SALTS_URL).json(&body).send().await {
            Ok(r) if r.status().is_success() => {
                log::debug!("gc: submitted match {}", salts.match_id);
                self.delivered += 1;
                true
            }
            Ok(r) => {
                log::warn!("gc salt post for {} failed: HTTP {}", salts.match_id, r.status());
                false
            }
            Err(e) => {
                log::warn!("gc salt post for {} failed: {e}", salts.match_id);
                false
            }
        }
    }

    fn now(&self) -> i64 {
        now_secs()
    }
}

async fn to_fetch(http: &reqwest::Client) -> Result<Vec<u64>, String> {
    let resp = http.get(TO_FETCH_URL).send().await.map_err(|e| e.to_string())?;
    resp.error_for_status().map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())
}

async fn run_account(
    ctx: &AuthContext,
    state: &mut AccountState,
    http: &reqwest::Client,
    stop: &AtomicBool,
    processed: &mut HashSet<u64>,
    last_request: &mut Option<Instant>,
) -> Result<u64, String> {
    let now = now_secs();
    if state.backed_off(now) {
        log::debug!("gc: account {} is backed off, skipping", ctx.account_id());
        return Ok(0);
    }
    let remaining = state.quota.remaining(now);
    if remaining == 0 {
        log::debug!("gc: account {} has used its 24h quota", ctx.account_id());
        return Ok(0);
    }
    let picks = fresh_newest_first(to_fetch(http).await?, processed, remaining);
    if picks.is_empty() {
        log::debug!("gc: nothing to fetch for account {}", ctx.account_id());
        return Ok(0);
    }
    log::debug!("gc: account {} fetching up to {} match(es)", ctx.account_id(), picks.len());

    // A failed handshake repeats identically on every pass, for example when the account does
    // not own the game, so the account is left alone for a day.
    let session = match GcSession::connect(ctx).await {
        Ok(s) => {
            state.backoff_until = None;
            s
        }
        Err(e) => {
            log::warn!("gc: account {} cannot reach the GC: {e}", ctx.account_id());
            state.backoff_until = Some(now + GC_BACKOFF_SECS);
            return Ok(0);
        }
    };
    let mut host = PassHost { stop, http, last_request, delivered: 0 };
    let outcome = fetch_loop(&session, &mut host, picks, state, processed).await;
    if outcome.rate_limited {
        log::warn!("gc: account {} was rate-limited by Steam", ctx.account_id());
    }
    log::debug!("gc: account {} delivered {} match(es)", ctx.account_id(), host.delivered);
    Ok(host.delivered)
}

async fn pass(stop: &AtomicBool, status: &Mutex<GcStatus>, app: &AppHandle, http: &reqwest::Client) {
    if dp_game::is_running() {
        log::debug!("gc: Deadlock is running, skipping this pass");
        return;
    }
    let Some(steam_dir) = dp_steam::steam_root() else {
        log::debug!("gc: Steam was not found, skipping this pass");
        status.lock_or_recover().last_error = Some("Steam was not found".into());
        return;
    };
    let contexts = match auth::recover_all(&steam_dir) {
        Ok(c) => c,
        Err(e) => {
            log::debug!("gc: no usable Steam session, skipping this pass: {e}");
            let mut st = status.lock_or_recover();
            st.accounts = 0;
            st.last_error = Some(e.to_string());
            return;
        }
    };
    log::debug!("gc: {} Steam account(s) available", contexts.len());
    status.lock_or_recover().accounts = u32::try_from(contexts.len()).unwrap_or(u32::MAX);

    let mut states = load_states(app);
    let mut processed = HashSet::new();
    let mut last_request = None;
    for ctx in &contexts {
        if stop.load(Ordering::Relaxed) || dp_game::is_running() {
            break;
        }
        let state = states.entry(ctx.steam_id64.to_string()).or_default();
        match run_account(ctx, state, http, stop, &mut processed, &mut last_request).await {
            Ok(n) => {
                let mut st = status.lock_or_recover();
                st.delivered += n;
                st.last_error = None;
            }
            Err(e) => {
                log::warn!("gc: account {} pass failed: {e}", ctx.account_id());
                status.lock_or_recover().last_error = Some(e);
            }
        }
        save_states(app, &states);
    }
}

fn run(stop: &AtomicBool, status: &Mutex<GcStatus>, app: &AppHandle, http: &reqwest::Client) {
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            log::warn!("gc: cannot build the async runtime: {e}");
            status.lock_or_recover().last_error = Some(e.to_string());
            return;
        }
    };
    while !stop.load(Ordering::Relaxed) {
        runtime.block_on(pass(stop, status, app, http));
        let mut waited = Duration::ZERO;
        while waited < PASS_INTERVAL && !stop.load(Ordering::Relaxed) {
            std::thread::sleep(POLL);
            waited += POLL;
        }
    }
}

pub mod commands {
    use tauri::{AppHandle, State};

    use super::{GcService, GcStatus};
    use crate::http::Http;

    #[tauri::command]
    pub fn set_gc_recovery_enabled(enabled: bool, app: AppHandle, gc: State<'_, GcService>, http: State<'_, Http>) {
        gc.set_enabled(enabled, app, http.0.clone());
    }

    #[tauri::command]
    pub fn gc_status(gc: State<'_, GcService>) -> GcStatus {
        gc.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovered_salts_are_tagged_with_the_account() {
        let s = to_ingest_salts(RecoveredSalts {
            match_id: 5,
            cluster_id: Some(404),
            metadata_salt: Some(1),
            replay_salt: None,
            account_id: 7,
        });
        assert_eq!(s.username, Some(7));
        assert_eq!((s.match_id, s.cluster_id, s.metadata_salt, s.replay_salt), (5, Some(404), Some(1), None));
        assert_eq!(serde_json::to_value(s).unwrap()["username"], "ingest-tool:7");
    }
}
