use crate::features::error::{error_codes, AppError};
use tauri::Manager;

error_codes! {
    pub enum KvError in "kv" {
        StoreFailed = "store_failed",
    }
}

fn dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path().app_data_dir().map_err(AppError::io)
}

fn store_failed(source: String) -> AppError {
    AppError::new(KvError::StoreFailed).detail(source)
}

pub mod commands {
    use super::{dir, store_failed};
    use crate::features::error::AppError;
    use dp_kv::KvStore;
    use serde_json::Value;
    use tauri::State;

    #[tauri::command]
    pub fn kv_get(
        app: tauri::AppHandle,
        kv: State<'_, KvStore>,
        store: String,
        key: String,
    ) -> Result<Option<Value>, AppError> {
        kv.get(&dir(&app)?, &store, &key).map_err(store_failed)
    }

    #[tauri::command]
    pub fn kv_set(
        app: tauri::AppHandle,
        kv: State<'_, KvStore>,
        store: String,
        key: String,
        value: Value,
    ) -> Result<(), AppError> {
        kv.set(&dir(&app)?, &store, &key, value).map_err(store_failed)
    }

    #[tauri::command]
    pub fn kv_delete(
        app: tauri::AppHandle,
        kv: State<'_, KvStore>,
        store: String,
        key: String,
    ) -> Result<(), AppError> {
        kv.delete(&dir(&app)?, &store, &key).map_err(store_failed)
    }
}

#[cfg(test)]
mod tests {
    use crate::features::error::assert_catalogued;
    use dp_kv::KvStore;
    use serde_json::json;

    #[test]
    fn every_kv_code_is_in_the_english_catalog() {
        assert_catalogued::<super::KvError>();
    }

    #[test]
    fn a_failed_store_access_is_coded_and_keeps_the_source_for_the_log() {
        let err = super::store_failed("disk full".into());
        assert_eq!(err.code(), "kv.store_failed");
        assert_eq!(err.to_string(), "kv.store_failed: disk full");
    }

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
