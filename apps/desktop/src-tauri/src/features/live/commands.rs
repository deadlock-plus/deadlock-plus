use tauri::{AppHandle, Manager};

use super::{LiveService, LiveState};

#[tauri::command]
pub async fn get_live_state(app: AppHandle) -> LiveState {
    app.state::<LiveService>().current()
}
