mod salts;
mod scan;
mod steam;

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use ts_rs::TS;

use notify::event::{CreateKind, ModifyKind};
use notify::{EventKind, RecursiveMode, Watcher};
use serde::Serialize;

use dp_sync::LockExt;
use salts::Salts;

const ENDPOINT: &str = "https://api.deadlock-api.com/v1/matches/salts";
const MAX_RETRIES: u32 = 10;
const RETRY_DELAY: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(500);

#[derive(Serialize, Clone, Default, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct IngestStatus {
    pub running: bool,
    pub steam_found: bool,
    pub submitted: u64,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct IngestService {
    stop: Mutex<Option<Arc<AtomicBool>>>,
    status: Arc<Mutex<IngestStatus>>,
}

impl IngestService {
    pub fn set_enabled(&self, enabled: bool, http: reqwest::Client) {
        let mut slot = self.stop.lock_or_recover();
        if let Some(flag) = slot.take() {
            flag.store(true, Ordering::Relaxed);
        }
        if !enabled {
            log::info!("ingest disabled");
            self.status.lock_or_recover().running = false;
            return;
        }
        log::info!("ingest enabled");

        let flag = Arc::new(AtomicBool::new(false));
        *slot = Some(flag.clone());
        let status = self.status.clone();
        status.lock_or_recover().running = true;
        std::thread::Builder::new()
            .name("ingest-watcher".into())
            .spawn(move || {
                run(&flag, &status, &http);
                if !flag.load(Ordering::Relaxed) {
                    status.lock_or_recover().running = false;
                }
            })
            .expect("spawn thread");
    }

    pub fn status(&self) -> IngestStatus {
        self.status.lock_or_recover().clone()
    }

    pub fn stop(&self) {
        if let Some(flag) = self.stop.lock_or_recover().take() {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

#[derive(Default)]
struct Seen(HashMap<u64, (bool, bool)>);

impl Seen {
    fn is_new(&self, s: &Salts) -> bool {
        let (meta, replay) = self.0.get(&s.match_id).copied().unwrap_or_default();
        (s.metadata_salt.is_some() && !meta) || (s.replay_salt.is_some() && !replay)
    }

    fn mark(&mut self, s: &Salts) {
        if self.0.len() > 10_000 {
            self.0.clear();
        }
        let entry = self.0.entry(s.match_id).or_default();
        entry.0 |= s.metadata_salt.is_some();
        entry.1 |= s.replay_salt.is_some();
    }
}

fn post(http: &reqwest::Client, salts: &[Salts], stop: &AtomicBool) -> Result<(), String> {
    let mut last = String::new();
    for attempt in 1..=MAX_RETRIES {
        match tauri::async_runtime::block_on(http.post(ENDPOINT).json(salts).send()) {
            Ok(r) if r.status().is_success() => return Ok(()),
            Ok(r) if r.status().as_u16() == 400 => {
                log::warn!("ingest batch of {} rejected by the API (400)", salts.len());
                return Err("rejected by the API (400)".into());
            }
            Ok(r) => last = format!("HTTP {}", r.status()),
            Err(e) => last = e.to_string(),
        }
        log::debug!("ingest attempt {attempt}/{MAX_RETRIES} failed: {last}");
        if attempt == MAX_RETRIES || stop.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(RETRY_DELAY);
    }
    log::warn!("giving up submitting {} match(es): {last}", salts.len());
    Err(last)
}

fn submit(http: &reqwest::Client, batch: &[Salts], seen: &mut Seen, status: &Mutex<IngestStatus>, stop: &AtomicBool) {
    let fresh: Vec<Salts> = batch.iter().copied().filter(|s| s.is_plausible() && seen.is_new(s)).collect();
    if fresh.is_empty() {
        return;
    }
    match post(http, &fresh, stop) {
        Ok(()) => {
            log::info!("submitted {} match(es) to the Deadlock API", fresh.len());
            fresh.iter().for_each(|s| seen.mark(s));
            let mut st = status.lock_or_recover();
            st.submitted += fresh.len() as u64;
            st.last_error = None;
        }
        Err(e) => status.lock_or_recover().last_error = Some(e),
    }
}

fn initial_scan(cache: &Path, steam_id: Option<u32>) -> Vec<Salts> {
    let mut urls = Vec::new();
    scan::scan_directory(cache, &mut urls);
    urls.iter().filter_map(|u| Salts::from_url(u, steam_id)).collect()
}

fn run(stop: &AtomicBool, status: &Mutex<IngestStatus>, http: &reqwest::Client) {
    let Some(cache) = steam::httpcache_dir() else {
        log::warn!("Steam http cache not found, ingest has nothing to watch");
        status.lock_or_recover().steam_found = false;
        return;
    };
    status.lock_or_recover().steam_found = true;

    let steam_id = steam::current_steam_id3();
    let mut seen = Seen::default();
    let initial = initial_scan(&cache, steam_id);
    log::debug!("ingest initial scan found {} candidate(s)", initial.len());
    submit(http, &initial, &mut seen, status, stop);

    let mut watch_failing = false;
    while !stop.load(Ordering::Relaxed) {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                if !watch_failing {
                    log::warn!("could not create the cache watcher, retrying: {e}");
                    watch_failing = true;
                }
                std::thread::sleep(RETRY_DELAY);
                continue;
            }
        };
        if let Err(e) = watcher.watch(&cache, RecursiveMode::Recursive) {
            if !watch_failing {
                log::warn!("could not watch the Steam http cache, retrying: {e}");
                watch_failing = true;
            }
            std::thread::sleep(RETRY_DELAY);
            continue;
        }
        if watch_failing {
            log::info!("cache watcher recovered");
            watch_failing = false;
        }

        while !stop.load(Ordering::Relaxed) {
            let Ok(Ok(event)) = rx.recv_timeout(POLL) else {
                continue;
            };
            let relevant = matches!(
                event.kind,
                EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(CreateKind::Any | CreateKind::File)
            );
            if !relevant {
                continue;
            }
            let batch: Vec<Salts> = event
                .paths
                .iter()
                .filter(|p| p.is_file())
                .filter_map(|p| scan::extract_replay_url(p))
                .filter_map(|u| Salts::from_url(&u, steam_id))
                .collect();
            submit(http, &batch, &mut seen, status, stop);
        }
    }
}

pub mod commands {
    use tauri::State;

    use super::{IngestService, IngestStatus};
    use crate::features::server_picker::ServerPickerState;

    #[tauri::command]
    pub fn set_ingest_enabled(enabled: bool, ingest: State<'_, IngestService>, picker: State<'_, ServerPickerState>) {
        ingest.set_enabled(enabled, picker.http.clone());
    }

    #[tauri::command]
    pub fn ingest_status(ingest: State<'_, IngestService>) -> IngestStatus {
        ingest.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn salts(id: u64, meta: bool) -> Salts {
        Salts {
            match_id: id,
            cluster_id: None,
            metadata_salt: meta.then_some(1),
            replay_salt: (!meta).then_some(1),
            username: None,
        }
    }

    #[test]
    fn metadata_and_replay_salts_are_tracked_separately() {
        let mut seen = Seen::default();
        assert!(seen.is_new(&salts(5, true)));
        seen.mark(&salts(5, true));
        assert!(!seen.is_new(&salts(5, true)));
        assert!(seen.is_new(&salts(5, false)));
    }
}
