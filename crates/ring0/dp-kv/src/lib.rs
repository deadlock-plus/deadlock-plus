use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use dp_versioned::{self, ReadError};

pub mod migrations;

/// The web view may only address these files, so a compromised page cannot pick an arbitrary path.
const STORES: &[&str] = &[
    "app-settings",
    "connection-settings",
    "server-picker-cache",
    "presets",
    "stats-cache",
    "frame-runs",
    "background-jobs",
    "gc-state",
    "postgame-matches",
];

type Entries = Map<String, Value>;

#[derive(Default)]
pub struct KvStore {
    loaded: Mutex<HashMap<String, Entries>>,
    /// Serialises writers so the disk write (with its fsync) can run without holding `loaded`,
    /// which would stall every read for as long as the disk takes.
    writing: Mutex<()>,
}

fn store_path(dir: &Path, store: &str) -> Result<PathBuf, String> {
    if !STORES.contains(&store) {
        return Err(format!("unknown store: {store}"));
    }
    Ok(dir.join(format!("{store}.json")))
}

impl KvStore {
    fn with_entries<R>(&self, dir: &Path, store: &str, f: impl FnOnce(&mut Entries) -> R) -> Result<R, String> {
        let path = store_path(dir, store)?;
        let mut loaded = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        if !loaded.contains_key(store) {
            let migrations = migrations::for_store(store);
            let entries = match dp_versioned::read::<Entries>(&path, migrations) {
                Ok(Some(entries)) => {
                    // Persist a migration's result now so it does not wait for the next write to this store.
                    if !migrations.is_empty() {
                        if let Err(e) = dp_versioned::write(&path, migrations, &entries) {
                            log::warn!("could not write {store}.json after loading: {e}");
                        }
                    }
                    entries
                }
                Ok(None) => Entries::new(),
                Err(e @ ReadError::Newer { .. }) => {
                    log::warn!("{store}.json is {e}; using defaults and leaving the file alone");
                    Entries::new()
                }
                Err(e) => {
                    log::warn!("{store}.json is {e}; starting empty");
                    Entries::new()
                }
            };
            loaded.insert(store.to_string(), entries);
        }
        Ok(f(loaded.get_mut(store).expect("inserted above")))
    }

    pub fn get(&self, dir: &Path, store: &str, key: &str) -> Result<Option<Value>, String> {
        self.with_entries(dir, store, |e| e.get(key).cloned())
    }

    pub fn set(&self, dir: &Path, store: &str, key: &str, value: Value) -> Result<(), String> {
        self.change(dir, store, |e| {
            e.insert(key.to_string(), value);
        })
    }

    pub fn delete(&self, dir: &Path, store: &str, key: &str) -> Result<(), String> {
        self.change(dir, store, |e| {
            e.remove(key);
        })
    }

    fn change(&self, dir: &Path, store: &str, f: impl FnOnce(&mut Entries)) -> Result<(), String> {
        let path = store_path(dir, store)?;
        let _writer = self.writing.lock().unwrap_or_else(|e| e.into_inner());
        let mut next = self.with_entries(dir, store, |entries| entries.clone())?;
        f(&mut next);
        dp_versioned::write(&path, migrations::for_store(store), &next).map_err(|e| {
            log::warn!("could not write {store}.json: {e}");
            e.to_string()
        })?;
        self.with_entries(dir, store, |entries| *entries = next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-kv-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    const SEEDED: &str = r#"{"schema_version":1,"data":{"matchHistory":[{"id":1}],"theme":"x","n":2}}"#;

    fn seeded(name: &str, file: &str, body: &str) -> PathBuf {
        let dir = temp_dir(name);
        std::fs::write(dir.join(file), body).unwrap();
        dir
    }

    #[test]
    fn match_history_is_dropped_from_connection_settings_and_siblings_stay() {
        let dir = seeded("mh-drop", "connection-settings.json", SEEDED);
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "connection-settings", "matchHistory").unwrap(), None);
        assert_eq!(kv.get(&dir, "connection-settings", "theme").unwrap(), Some(json!("x")));
        assert_eq!(kv.get(&dir, "connection-settings", "n").unwrap(), Some(json!(2)));
    }

    #[test]
    fn the_stored_file_loses_match_history_on_first_load() {
        let dir = seeded("mh-disk", "connection-settings.json", SEEDED);
        KvStore::default().get(&dir, "connection-settings", "theme").unwrap();
        let raw: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("connection-settings.json")).unwrap()).unwrap();
        assert_eq!(raw, json!({"data": {"theme": "x", "n": 2}, "schema_version": 2}));
    }

    #[test]
    fn match_history_survives_in_other_stores() {
        let dir = seeded("mh-other", "presets.json", SEEDED);
        assert_eq!(KvStore::default().get(&dir, "presets", "matchHistory").unwrap(), Some(json!([{"id": 1}])));
    }

    #[test]
    fn a_second_load_changes_nothing() {
        let dir = seeded("mh-twice", "connection-settings.json", SEEDED);
        KvStore::default().get(&dir, "connection-settings", "theme").unwrap();
        let first = std::fs::read_to_string(dir.join("connection-settings.json")).unwrap();
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "connection-settings", "theme").unwrap(), Some(json!("x")));
        kv.set(&dir, "connection-settings", "n", json!(2)).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("connection-settings.json")).unwrap(), first);
    }

    #[test]
    fn a_legacy_flat_file_is_cleaned_too() {
        let dir = seeded("mh-legacy", "connection-settings.json", r#"{"matchHistory":"corrupt","keep":1}"#);
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "connection-settings", "matchHistory").unwrap(), None);
        assert_eq!(kv.get(&dir, "connection-settings", "keep").unwrap(), Some(json!(1)));
    }

    #[test]
    fn an_absent_store_file_is_not_created_by_loading() {
        let dir = temp_dir("mh-absent");
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "connection-settings", "matchHistory").unwrap(), None);
        assert!(!dir.join("connection-settings.json").exists());
    }

    #[test]
    fn a_store_without_the_key_is_unchanged_in_content() {
        let dir = seeded("mh-nokey", "connection-settings.json", r#"{"schema_version":1,"data":{"a":1}}"#);
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "connection-settings", "a").unwrap(), Some(json!(1)));
        kv.set(&dir, "connection-settings", "b", json!(2)).unwrap();
        assert_eq!(KvStore::default().get(&dir, "connection-settings", "a").unwrap(), Some(json!(1)));
    }

    #[test]
    fn the_backend_stores_are_addressable() {
        let dir = temp_dir("backend-stores");
        let kv = KvStore::default();
        for store in ["gc-state", "postgame-matches"] {
            kv.set(&dir, store, "k", json!(1)).unwrap_or_else(|e| panic!("{store}: {e}"));
        }
    }

    #[test]
    fn set_then_get_persists_across_a_fresh_store() {
        let dir = temp_dir("persist");
        KvStore::default().set(&dir, "app-settings", "theme", json!("midnight")).unwrap();
        assert_eq!(KvStore::default().get(&dir, "app-settings", "theme").unwrap(), Some(json!("midnight")));
    }

    #[test]
    fn concurrent_writers_all_land() {
        let dir = temp_dir("concurrent");
        let kv = std::sync::Arc::new(KvStore::default());
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let (kv, dir) = (kv.clone(), dir.clone());
                std::thread::spawn(move || kv.set(&dir, "presets", &format!("k{i}"), json!(i)).unwrap())
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        let fresh = KvStore::default();
        for i in 0..8 {
            assert_eq!(fresh.get(&dir, "presets", &format!("k{i}")).unwrap(), Some(json!(i)));
        }
    }

    #[test]
    fn delete_removes_the_key_on_disk() {
        let dir = temp_dir("delete");
        let kv = KvStore::default();
        kv.set(&dir, "presets", "presets", json!([1])).unwrap();
        kv.delete(&dir, "presets", "presets").unwrap();
        assert_eq!(KvStore::default().get(&dir, "presets", "presets").unwrap(), None);
    }

    #[test]
    fn unknown_stores_are_rejected() {
        let dir = temp_dir("unknown");
        let kv = KvStore::default();
        assert!(kv.get(&dir, "../evil", "k").is_err());
        assert!(kv.set(&dir, "other", "k", json!(1)).is_err());
        assert!(!dir.join("other.json").exists());
    }

    #[test]
    fn reads_the_flat_file_the_old_store_plugin_wrote() {
        let dir = temp_dir("legacy");
        std::fs::write(dir.join("app-settings.json"), r#"{"theme":"daylight","closeToTray":true}"#).unwrap();
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "app-settings", "theme").unwrap(), Some(json!("daylight")));
        kv.set(&dir, "app-settings", "motion", json!("reduce")).unwrap();
        assert_eq!(KvStore::default().get(&dir, "app-settings", "closeToTray").unwrap(), Some(json!(true)));
    }

    #[test]
    fn a_newer_file_is_left_alone_and_writes_fail() {
        let dir = temp_dir("newer");
        let original = r#"{"schema_version":9,"data":{"theme":"future"}}"#;
        std::fs::write(dir.join("app-settings.json"), original).unwrap();
        let kv = KvStore::default();
        assert_eq!(kv.get(&dir, "app-settings", "theme").unwrap(), None);
        assert!(kv.set(&dir, "app-settings", "theme", json!("x")).is_err());
        assert_eq!(kv.get(&dir, "app-settings", "theme").unwrap(), None);
        assert_eq!(std::fs::read_to_string(dir.join("app-settings.json")).unwrap(), original);
    }

    #[test]
    fn a_failed_write_does_not_change_what_get_returns() {
        let dir = temp_dir("failed-write");
        let kv = KvStore::default();
        kv.set(&dir, "presets", "a", json!(1)).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        std::fs::write(&dir, b"a file where the folder was").unwrap();
        assert!(kv.set(&dir, "presets", "a", json!(2)).is_err());
        assert_eq!(kv.get(&dir, "presets", "a").unwrap(), Some(json!(1)));
    }
}
