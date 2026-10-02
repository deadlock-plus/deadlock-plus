use dp_postgame::StoredMatch;
use tauri::{AppHandle, Manager};

use super::PostgameService;

#[tauri::command]
pub async fn set_postgame_capture_enabled(enabled: bool, app: AppHandle) {
    app.state::<PostgameService>().set_enabled(enabled, &app);
}

#[tauri::command]
pub async fn get_postgame_matches(account_id: u32, app: AppHandle) -> Vec<StoredMatch> {
    app.state::<PostgameService>().matches(&app, account_id)
}

#[tauri::command]
pub async fn reconcile_postgame_matches(account_id: u32, api_match_ids: Vec<u64>, app: AppHandle) -> usize {
    app.state::<PostgameService>().reconcile(&app, account_id, &api_match_ids)
}
