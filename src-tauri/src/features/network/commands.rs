use tauri::State;
#[cfg(windows)]
use tauri::{AppHandle, Manager};

use super::types::{HistoryPoint, Snapshot};
use super::NetworkMonitor;
use crate::features::server_picker::ServerPickerState;

#[cfg(windows)]
const HISTORY_FILE: &str = "connection-history.jsonl";

#[cfg(windows)]
pub fn start_monitor(app: &AppHandle) {
    let dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("network monitor not started, no app data dir: {e}");
            return;
        }
    };
    let http = app.state::<ServerPickerState>().http.clone();
    app.state::<NetworkMonitor>().start(http, dir.join(HISTORY_FILE));
}

#[cfg(windows)]
#[tauri::command]
pub fn start_network_monitor(app: AppHandle) {
    start_monitor(&app);
}

#[cfg(windows)]
#[tauri::command]
pub fn network_snapshot(monitor: State<'_, NetworkMonitor>) -> Snapshot {
    monitor.snapshot()
}

#[cfg(windows)]
#[tauri::command]
pub fn network_history(monitor: State<'_, NetworkMonitor>) -> Vec<HistoryPoint> {
    monitor.history()
}

#[cfg(not(windows))]
#[tauri::command]
pub fn start_network_monitor(_monitor: State<'_, NetworkMonitor>, _picker: State<'_, ServerPickerState>) {}

#[cfg(not(windows))]
pub fn start_monitor(_app: &tauri::AppHandle) {}

#[cfg(not(windows))]
#[tauri::command]
pub fn network_snapshot(_monitor: State<'_, NetworkMonitor>) -> Snapshot {
    Snapshot::default()
}

#[cfg(not(windows))]
#[tauri::command]
pub fn network_history(_monitor: State<'_, NetworkMonitor>) -> Vec<HistoryPoint> {
    Vec::new()
}
