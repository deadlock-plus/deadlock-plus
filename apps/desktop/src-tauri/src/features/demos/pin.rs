pub mod commands {
    use dp_demos::pin::PinStore;
    use tauri::{Manager, State};

    fn dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
        app.path().app_data_dir().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn list_pinned(app: tauri::AppHandle, store: State<'_, PinStore>) -> Result<Vec<u64>, String> {
        Ok(store.snapshot(&dir(&app)?).ids())
    }

    #[tauri::command]
    pub fn set_pinned(
        app: tauri::AppHandle,
        store: State<'_, PinStore>,
        match_id: u64,
        pinned: bool,
    ) -> Result<Vec<u64>, String> {
        let pins = store.update(&dir(&app)?, |p| {
            if pinned {
                p.pin(match_id);
            } else {
                p.unpin(match_id);
            }
        })?;
        log::debug!("replay {match_id} {}", if pinned { "pinned" } else { "unpinned" });
        Ok(pins.ids())
    }
}
