use dp_postgame::StoredMatch;
use tauri::{AppHandle, Manager};

use super::PostgameService;

#[tauri::command]
pub async fn get_postgame_matches(account_id: u32, app: AppHandle) -> Vec<StoredMatch> {
    app.state::<PostgameService>().matches(&app, account_id)
}

#[tauri::command]
pub async fn reconcile_postgame_matches(account_id: u32, api_match_ids: Vec<u64>, app: AppHandle) -> usize {
    app.state::<PostgameService>().reconcile(&app, account_id, &api_match_ids)
}

#[tauri::command]
pub async fn get_postgame_detail(match_id: u64, app: AppHandle) -> Option<serde_json::Value> {
    let account_id = dp_steam::current_steam_id32()?;
    tauri::async_runtime::spawn_blocking(move || app.state::<PostgameService>().detail(&app, account_id, match_id))
        .await
        .ok()
        .flatten()
}
