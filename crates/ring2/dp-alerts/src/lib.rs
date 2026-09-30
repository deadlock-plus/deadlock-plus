pub mod feed;

use std::collections::HashSet;
use std::path::Path;

use dp_versioned::{self, Migration};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const STORE_FILE: &str = "alerts.json";
const MAX_ALERTS: usize = 50;
const MAX_SEEN: usize = 500;
const MIGRATIONS: &[Migration] = &[];
/// Bumped when stored alerts gain fields (so an older file is listed afresh rather than shown
/// half-empty) or when `published` is computed differently (so already-cached entries pick up the
/// new value): `merge` never revisits an already-`seen` id, so a parsing fix alone never reaches
/// what's already on disk — only a full relist does.
const STORE_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub link: String,
    /// "forum" or "steam", as reported by the feed.
    pub source: String,
    /// ISO 8601 UTC, so plain string order is chronological order.
    pub published: String,
    /// The update type from the title, e.g. "Minor Update" or "Patch notes".
    pub kind: String,
    pub summary: String,
    /// Steam-hosted art from the post body, when it has any.
    pub image: Option<String>,
    pub read: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stored {
    version: u32,
    /// False until the first successful poll, which lists what exists as already read, without alerting.
    listed: bool,
    seen: Vec<String>,
    pub alerts: Vec<Alert>,
}

/// Folds a fetched feed into the stored state and returns the alerts that are new, newest first.
pub fn merge(stored: &mut Stored, items: Vec<Alert>) -> Vec<Alert> {
    if !stored.listed {
        let mut existing = items;
        existing.sort_by(|a, b| b.published.cmp(&a.published));
        existing.truncate(MAX_ALERTS);
        existing.iter_mut().for_each(|a| a.read = true);
        stored.seen = existing.iter().map(|a| a.id.clone()).collect();
        stored.alerts = existing;
        stored.listed = true;
        stored.version = STORE_VERSION;
        return Vec::new();
    }

    let known: HashSet<&str> = stored.seen.iter().map(String::as_str).collect();
    let mut fresh: Vec<Alert> = Vec::new();
    for item in items {
        if !known.contains(item.id.as_str()) && !fresh.iter().any(|f| f.id == item.id) {
            fresh.push(item);
        }
    }
    fresh.sort_by(|a, b| b.published.cmp(&a.published));

    stored.seen.extend(fresh.iter().map(|a| a.id.clone()));
    if stored.seen.len() > MAX_SEEN {
        stored.seen.drain(..stored.seen.len() - MAX_SEEN);
    }
    stored.alerts.splice(0..0, fresh.iter().cloned());
    stored.alerts.truncate(MAX_ALERTS);
    fresh
}

/// `Stored::version` is separate from the file's schema version: a mismatch means the feed parsing
/// changed and the list must be rebuilt from the next poll, not migrated.
pub fn load(path: &Path) -> Stored {
    match dp_versioned::read::<Stored>(path, MIGRATIONS) {
        Ok(Some(s)) if s.version == STORE_VERSION => s,
        Ok(Some(s)) => {
            log::info!("{STORE_FILE} is version {}, relisting alerts from scratch", s.version);
            Stored::default()
        }
        Ok(None) => Stored::default(),
        Err(e) => {
            log::warn!("could not read {STORE_FILE}, starting empty: {e}");
            Stored::default()
        }
    }
}

pub fn save(path: &Path, stored: &Stored) -> std::io::Result<()> {
    dp_versioned::write(path, MIGRATIONS, stored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn alert(id: &str, published: &str) -> Alert {
        Alert {
            id: id.into(),
            title: format!("title {id}"),
            link: format!("https://example.test/{id}"),
            source: "forum".into(),
            published: published.into(),
            kind: "Patch notes".into(),
            summary: String::new(),
            image: None,
            read: false,
        }
    }

    #[test]
    fn first_poll_lists_existing_items_as_read_without_reporting_them() {
        let mut s = Stored::default();
        let fresh = merge(&mut s, vec![alert("a", "2026-09-01"), alert("b", "2026-09-02")]);
        assert!(fresh.is_empty());
        assert!(s.listed);
        assert_eq!(s.alerts.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        assert!(s.alerts.iter().all(|a| a.read));
        assert_eq!(s.seen.len(), 2);
    }

    #[test]
    fn first_poll_list_is_capped() {
        let mut s = Stored::default();
        let items: Vec<Alert> =
            (0..MAX_ALERTS + 5).map(|i| alert(&format!("id{i}"), &format!("2026-01-{i:04}"))).collect();
        merge(&mut s, items);
        assert_eq!(s.alerts.len(), MAX_ALERTS);
    }

    #[test]
    fn items_after_the_first_poll_are_unread() {
        let mut s = Stored::default();
        merge(&mut s, vec![alert("a", "2026-09-01")]);
        merge(&mut s, vec![alert("a", "2026-09-01"), alert("b", "2026-09-02")]);
        assert!(!s.alerts[0].read);
        assert!(s.alerts[1].read);
    }

    #[test]
    fn later_polls_report_only_unseen_items_newest_first() {
        let mut s = Stored::default();
        merge(&mut s, vec![alert("a", "2026-09-01")]);
        let fresh = merge(&mut s, vec![alert("a", "2026-09-01"), alert("b", "2026-09-02"), alert("c", "2026-09-03")]);
        assert_eq!(fresh.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["c", "b"]);
        assert_eq!(s.alerts.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(), ["c", "b", "a"]);
    }

    #[test]
    fn an_item_is_reported_once() {
        let mut s = Stored::default();
        merge(&mut s, vec![]);
        assert_eq!(merge(&mut s, vec![alert("a", "2026-09-01")]).len(), 1);
        assert!(merge(&mut s, vec![alert("a", "2026-09-01")]).is_empty());
    }

    #[test]
    fn duplicate_ids_within_one_feed_count_once() {
        let mut s = Stored::default();
        merge(&mut s, vec![]);
        assert_eq!(merge(&mut s, vec![alert("a", "2026-09-01"), alert("a", "2026-09-01")]).len(), 1);
    }

    #[test]
    fn stored_lists_are_capped() {
        let mut s = Stored::default();
        merge(&mut s, vec![]);
        let items: Vec<Alert> =
            (0..MAX_SEEN + 10).map(|i| alert(&format!("id{i}"), &format!("2026-01-{i:04}"))).collect();
        merge(&mut s, items);
        assert_eq!(s.alerts.len(), MAX_ALERTS);
        assert_eq!(s.seen.len(), MAX_SEEN);
    }

    #[test]
    fn state_round_trips_through_disk_and_tolerates_corruption() {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-alerts", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join(STORE_FILE);

        assert!(!load(&path).listed);

        let mut s = Stored::default();
        merge(&mut s, vec![alert("a", "2026-09-01")]);
        save(&path, &s).unwrap();
        let back = load(&path);
        assert!(back.listed);
        assert_eq!(back.seen, ["a"]);

        let mut old: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        old["data"]["version"] = 1.into();
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        assert!(!load(&path).listed);
        save(&path, &s).unwrap();

        fs::write(&path, "not json").unwrap();
        assert!(!load(&path).listed);
        let _ = fs::remove_dir_all(&dir);
    }
}
