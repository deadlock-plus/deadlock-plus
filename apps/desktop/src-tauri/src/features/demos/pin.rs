use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::delete::Failure;
use super::parse_demo_filename;
use dp_versioned::{self, Migration};

const MIGRATIONS: &[Migration] = &[];

/// Pinned replays by match id. Stored outside the replays folder so the game and Steam never
/// see it.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pins(BTreeSet<u64>);

impl Pins {
    pub fn pin(&mut self, match_id: u64) {
        self.0.insert(match_id);
    }

    pub fn unpin(&mut self, match_id: u64) {
        self.0.remove(&match_id);
    }

    pub fn is_pinned(&self, match_id: u64) -> bool {
        self.0.contains(&match_id)
    }

    pub fn ids(&self) -> Vec<u64> {
        self.0.iter().copied().collect()
    }
}

/// Splits requested file names into the ones that may be deleted and refusals for pinned replays.
/// Names that are not replay files pass through so the normal delete guard reports them.
pub fn split_pinned(names: &[String], pins: &Pins) -> (Vec<String>, Vec<Failure>) {
    let (mut allowed, mut refused) = (Vec::new(), Vec::new());
    for name in names {
        match parse_demo_filename(name) {
            Some((id, _)) if pins.is_pinned(id) => refused.push(Failure {
                file_name: name.clone(),
                message: "Pinned replays can't be deleted. Unpin them first.".into(),
            }),
            _ => allowed.push(name.clone()),
        }
    }
    (allowed, refused)
}

/// A corrupt file reads as empty; the next save replaces it.
pub fn load(path: &Path) -> Pins {
    dp_versioned::read(path, MIGRATIONS)
        .unwrap_or_else(|e| {
            log::warn!("could not read the pin list, starting empty: {e}");
            None
        })
        .unwrap_or_default()
}

pub fn save(path: &Path, pins: &Pins) -> std::io::Result<()> {
    dp_versioned::write(path, MIGRATIONS, pins)
}

#[derive(Default)]
pub struct PinStore(Mutex<Option<Pins>>);

impl PinStore {
    fn path(dir: &Path) -> PathBuf {
        dir.join("demo-pins.json")
    }

    pub fn snapshot(&self, dir: &Path) -> Pins {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        guard.get_or_insert_with(|| load(&Self::path(dir))).clone()
    }

    /// The change is kept in memory only if it reached the disk.
    pub fn update(&self, dir: &Path, f: impl FnOnce(&mut Pins)) -> Result<Pins, String> {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let current = guard.get_or_insert_with(|| load(&Self::path(dir)));
        let mut next = current.clone();
        f(&mut next);
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        save(&Self::path(dir), &next).map_err(|e| {
            log::error!("could not save the pin list: {e}");
            e.to_string()
        })?;
        *current = next.clone();
        Ok(next)
    }
}

pub mod commands {
    use super::PinStore;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-pin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn pinning_is_idempotent_and_unpinning_forgets() {
        let mut p = Pins::default();
        p.pin(7);
        p.pin(7);
        assert!(p.is_pinned(7));
        assert_eq!(p.ids(), vec![7]);
        p.unpin(7);
        assert!(!p.is_pinned(7));
        p.unpin(7);
        assert!(p.ids().is_empty());
    }

    #[test]
    fn ids_come_back_sorted() {
        let mut p = Pins::default();
        p.pin(9);
        p.pin(2);
        assert_eq!(p.ids(), vec![2, 9]);
    }

    #[test]
    fn pinned_replays_are_refused_for_both_file_kinds() {
        let mut p = Pins::default();
        p.pin(1);
        p.pin(2);
        let (allowed, refused) = split_pinned(&names(&["1.dem", "2.dem.partial", "3.dem", "notes.txt"]), &p);
        assert_eq!(allowed, names(&["3.dem", "notes.txt"]));
        let refused: Vec<_> = refused.iter().map(|f| f.file_name.as_str()).collect();
        assert_eq!(refused, vec!["1.dem", "2.dem.partial"]);
    }

    #[test]
    fn the_refusal_tells_the_user_what_to_do() {
        let mut p = Pins::default();
        p.pin(1);
        let (_, refused) = split_pinned(&names(&["1.dem"]), &p);
        assert_eq!(refused[0].message, "Pinned replays can't be deleted. Unpin them first.");
    }

    #[test]
    fn pins_survive_a_save_and_load() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("demo-pins.json");
        let mut p = Pins::default();
        p.pin(42);
        save(&path, &p).unwrap();
        assert_eq!(load(&path), p);
    }

    #[test]
    fn a_missing_or_corrupt_file_loads_as_empty() {
        let dir = temp_dir("corrupt");
        assert_eq!(load(&dir.join("nope.json")), Pins::default());
        std::fs::write(dir.join("bad.json"), b"{ not json").unwrap();
        assert_eq!(load(&dir.join("bad.json")), Pins::default());
    }

    #[test]
    fn a_failed_save_does_not_change_what_the_store_reports() {
        let dir = temp_dir("failsave");
        let store = PinStore::default();
        store.update(&dir, |p| p.pin(1)).unwrap();
        std::fs::create_dir_all(dir.join("demo-pins.json.tmp")).unwrap();
        assert!(store.update(&dir, |p| p.pin(2)).is_err());
        assert!(!store.snapshot(&dir).is_pinned(2));
        assert!(store.snapshot(&dir).is_pinned(1));
    }
}
