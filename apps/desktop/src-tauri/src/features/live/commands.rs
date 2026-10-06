use tauri::{AppHandle, Manager};

use super::board::LiveMatch;
use super::{LiveService, LiveState};

#[tauri::command]
pub async fn get_live_state(app: AppHandle) -> LiveState {
    app.state::<LiveService>().current()
}

#[tauri::command]
pub async fn get_live_match(app: AppHandle) -> LiveMatch {
    app.state::<LiveService>().current_match()
}
