mod error;

pub mod commands {
    use super::error::{check_clear, StorageError};
    use crate::features::error::AppError;
    use dp_storage::{
        clear, entry_stats, is_clearable, location, ClearReport, EntryId, EntryInfo, EntryStats, Roots, ALL_ENTRIES,
    };
    use std::path::PathBuf;

    fn current_roots(app: &tauri::AppHandle) -> Roots {
        use tauri::Manager;
        Roots {
            install: dp_steam::game_install_dir(),
            userdata: dp_steam::current_account().and_then(|a| a.userdata_dir).map(PathBuf::from),
            app_data: app.path().app_data_dir().ok(),
            logs: app.path().app_log_dir().ok(),
        }
    }

    #[tauri::command]
    pub fn storage_entries(app: tauri::AppHandle) -> Vec<EntryInfo> {
        let roots = current_roots(&app);
        ALL_ENTRIES
            .into_iter()
            .map(|id| EntryInfo {
                id,
                path: location(id, &roots).filter(|p| p.exists()).map(|p| p.to_string_lossy().into_owned()),
                clearable: is_clearable(id),
            })
            .collect()
    }

    #[tauri::command]
    pub async fn storage_entry_stats(app: tauri::AppHandle, id: EntryId) -> Result<EntryStats, AppError> {
        let roots = current_roots(&app);
        tauri::async_runtime::spawn_blocking(move || entry_stats(id, &roots)).await.map_err(AppError::internal)
    }

    /// The frontend sends an entry id, never a path.
    #[tauri::command]
    pub fn storage_reveal(app: tauri::AppHandle, id: EntryId) -> Result<(), AppError> {
        let path = location(id, &current_roots(&app)).filter(|p| p.exists()).ok_or(StorageError::LocationMissing)?;
        crate::features::reveal::show(&path).map_err(|e| {
            log::error!("could not reveal a storage location ({id:?}): {e}");
            AppError::new(StorageError::RevealFailed).detail(e)
        })
    }

    #[tauri::command]
    pub async fn storage_clear(app: tauri::AppHandle, id: EntryId) -> Result<ClearReport, AppError> {
        check_clear(id, dp_game::is_running()).inspect_err(|e| log::warn!("storage clear refused ({id:?}): {e}"))?;
        let roots = current_roots(&app);
        tauri::async_runtime::spawn_blocking(move || clear(id, &roots))
            .await
            .map_err(AppError::internal)?
            .map_err(AppError::internal)
    }
}
