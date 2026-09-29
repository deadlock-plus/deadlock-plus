use tauri::Manager;

fn dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

pub mod commands {
    use super::dir;
    use dp_kv::KvStore;
    use serde_json::Value;
    use tauri::State;

    #[tauri::command]
    pub fn kv_get(
        app: tauri::AppHandle,
        kv: State<'_, KvStore>,
        store: String,
        key: String,
    ) -> Result<Option<Value>, String> {
        kv.get(&dir(&app)?, &store, &key)
    }

    #[tauri::command]
    pub fn kv_set(
        app: tauri::AppHandle,
        kv: State<'_, KvStore>,
        store: String,
        key: String,
        value: Value,
    ) -> Result<(), String> {
        kv.set(&dir(&app)?, &store, &key, value)
    }

    #[tauri::command]
    pub fn kv_delete(app: tauri::AppHandle, kv: State<'_, KvStore>, store: String, key: String) -> Result<(), String> {
        kv.delete(&dir(&app)?, &store, &key)
    }
}

#[cfg(test)]
mod tests {
    use dp_kv::KvStore;
    use serde_json::json;

    #[test]
    fn the_background_job_settings_store_is_writable() {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-kv-jobs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = crate::features::jobs::STORE;
        KvStore::default().set(&dir, store, "pauseInGame", json!(false)).unwrap();
        assert_eq!(KvStore::default().get(&dir, store, "pauseInGame").unwrap(), Some(json!(false)));
    }
}
