use tauri::{AppHandle, Manager, State};

use super::types::{HistoryPoint, Snapshot};
use super::NetworkMonitor;
use crate::http::Http;

const HISTORY_FILE: &str = "connection-history.jsonl";

pub fn start_monitor(app: &AppHandle, prompt: bool) {
    let dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("network monitor not started, no app data dir: {e}");
            return;
        }
    };
    let http = app.state::<Http>().0.clone();
    app.state::<NetworkMonitor>().start(http, dir.join(HISTORY_FILE), prompt);
}

#[tauri::command]
pub fn start_network_monitor(app: AppHandle, allow_prompt: bool) {
    start_monitor(&app, allow_prompt);
}

#[tauri::command]
pub fn network_snapshot(monitor: State<'_, NetworkMonitor>) -> Snapshot {
    monitor.snapshot()
}

#[tauri::command]
pub fn network_history(monitor: State<'_, NetworkMonitor>) -> Vec<HistoryPoint> {
    monitor.history()
}
