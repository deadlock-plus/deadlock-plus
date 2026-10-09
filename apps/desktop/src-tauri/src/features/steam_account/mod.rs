pub mod commands {
    use crate::features::error::AppError;
    use dp_steam::SteamAccount;

    #[tauri::command]
    pub async fn current_steam_account() -> Result<Option<SteamAccount>, AppError> {
        tauri::async_runtime::spawn_blocking(dp_steam::current_account).await.map_err(AppError::internal)
    }

    #[tauri::command]
    pub async fn local_steam_account_ids() -> Result<Vec<u32>, AppError> {
        tauri::async_runtime::spawn_blocking(dp_steam::local_account_ids).await.map_err(AppError::internal)
    }
}
