use std::path::Path;
use ts_rs::TS;

use serde::{Deserialize, Serialize};

use super::pin::Pins;
use dp_versioned::{self, Migration};

use super::{DemoEntry, DemoStatus};

const DAY_MS: u64 = 24 * 60 * 60 * 1000;
const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CleanupRuleKind")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuleKind {
    OlderThanDays { days: u32 },
    LargerThanMb { mb: u64 },
    Outdated,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, rename = "CleanupRule")]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: u32,
    pub enabled: bool,
    #[serde(flatten)]
    pub kind: RuleKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CleanupMatch {
    pub file_name: String,
    pub match_id: u64,
    pub size: u64,
    pub rule_ids: Vec<u32>,
}

/// A zero threshold would match every replay, so it is rejected instead of clamped.
pub fn validate(rules: &[Rule]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for rule in rules {
        if !seen.insert(rule.id) {
            return Err("Two cleanup rules share an id.".into());
        }
        match rule.kind {
            RuleKind::OlderThanDays { days: 0 } => return Err("The age limit must be at least 1 day.".into()),
            RuleKind::LargerThanMb { mb: 0 } => return Err("The size limit must be more than 0.".into()),
            _ => {}
        }
    }
    Ok(())
}

pub fn rule_matches(kind: &RuleKind, demo: &DemoEntry, now_ms: u64) -> bool {
    match kind {
        // An unreadable modified time is stored as 0; treating it as "very old" would offer the
        // replay for deletion on no evidence.
        RuleKind::OlderThanDays { days } => {
            demo.modified_ms > 0 && now_ms.saturating_sub(demo.modified_ms) > u64::from(*days) * DAY_MS
        }
        RuleKind::LargerThanMb { mb } => demo.size > mb.saturating_mul(MB),
        RuleKind::Outdated => demo.status == DemoStatus::Outdated,
        RuleKind::Partial => demo.status == DemoStatus::Partial,
    }
}

/// Pinned replays are never returned, whatever the rules say.
pub fn select(rules: &[Rule], demos: &[DemoEntry], pins: &Pins, now_ms: u64) -> Vec<CleanupMatch> {
    demos
        .iter()
        .filter(|d| !pins.is_pinned(d.match_id))
        .filter_map(|d| {
            let rule_ids: Vec<u32> =
                rules.iter().filter(|r| r.enabled && rule_matches(&r.kind, d, now_ms)).map(|r| r.id).collect();
            (!rule_ids.is_empty()).then(|| CleanupMatch {
                file_name: d.file_name.clone(),
                match_id: d.match_id,
                size: d.size,
                rule_ids,
            })
        })
        .collect()
}

const MIGRATIONS: &[Migration] = &[];

/// A missing or unreadable file means no rules.
pub fn load_rules(path: &Path) -> Vec<Rule> {
    dp_versioned::read(path, MIGRATIONS)
        .unwrap_or_else(|e| {
            log::warn!("could not read the cleanup rules, using none: {e}");
            None
        })
        .unwrap_or_default()
}

pub fn save_rules(path: &Path, rules: &[Rule]) -> std::io::Result<()> {
    dp_versioned::write(path, MIGRATIONS, &rules)
}

pub mod commands {
    use super::{load_rules, save_rules, select, validate, CleanupMatch, Rule};
    use crate::features::demos::pin::PinStore;
    use crate::features::demos::{list_demos_in, replays_dir};
    use tauri::{Manager, State};

    fn app_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
        app.path().app_data_dir().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn list_cleanup_rules(app: tauri::AppHandle) -> Result<Vec<Rule>, String> {
        Ok(load_rules(&app_dir(&app)?.join("demo-cleanup-rules.json")))
    }

    #[tauri::command]
    pub fn save_cleanup_rules(app: tauri::AppHandle, rules: Vec<Rule>) -> Result<(), String> {
        validate(&rules)?;
        let dir = app_dir(&app)?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        save_rules(&dir.join("demo-cleanup-rules.json"), &rules).map_err(|e| {
            log::error!("could not save the cleanup rules: {e}");
            e.to_string()
        })?;
        log::info!("cleanup rules saved ({} rules)", rules.len());
        Ok(())
    }

    /// Only a preview: nothing is deleted here. The frontend sends the chosen file names through
    /// the normal delete flow, which applies its own guards.
    #[tauri::command]
    pub async fn cleanup_matches(
        app: tauri::AppHandle,
        pins: State<'_, PinStore>,
    ) -> Result<Vec<CleanupMatch>, String> {
        let dir = app_dir(&app)?;
        let pins = pins.snapshot(&dir);
        tauri::async_runtime::spawn_blocking(move || {
            let Some(replays) = replays_dir() else {
                return Vec::new();
            };
            let now_ms =
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
            let rules = load_rules(&dir.join("demo-cleanup-rules.json"));
            let matches = select(&rules, &list_demos_in(&replays), &pins, now_ms);
            log::info!("cleanup preview: {} replays match {} rules", matches.len(), rules.len());
            matches
        })
        .await
        .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 100 * DAY_MS;

    fn demo(id: u64, size: u64, age_days: u64, status: DemoStatus) -> DemoEntry {
        DemoEntry {
            match_id: id,
            file_name: format!("{id}.dem"),
            size,
            modified_ms: NOW - age_days * DAY_MS,
            status,
            build_num: None,
        }
    }

    fn rule(id: u32, kind: RuleKind) -> Rule {
        Rule { id, enabled: true, kind }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-cleanup-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ids(matches: &[CleanupMatch]) -> Vec<u64> {
        matches.iter().map(|m| m.match_id).collect()
    }

    #[test]
    fn older_than_is_strictly_older_and_ignores_unknown_dates() {
        let kind = RuleKind::OlderThanDays { days: 30 };
        assert!(!rule_matches(&kind, &demo(1, 1, 30, DemoStatus::Complete), NOW));
        assert!(rule_matches(&kind, &demo(1, 1, 31, DemoStatus::Complete), NOW));
        let mut unknown = demo(1, 1, 0, DemoStatus::Complete);
        unknown.modified_ms = 0;
        assert!(!rule_matches(&kind, &unknown, NOW));
    }

    #[test]
    fn a_future_modified_time_is_not_old() {
        let mut d = demo(1, 1, 0, DemoStatus::Complete);
        d.modified_ms = NOW + DAY_MS;
        assert!(!rule_matches(&RuleKind::OlderThanDays { days: 1 }, &d, NOW));
    }

    #[test]
    fn larger_than_is_strictly_larger() {
        let kind = RuleKind::LargerThanMb { mb: 100 };
        assert!(!rule_matches(&kind, &demo(1, 100 * MB, 0, DemoStatus::Complete), NOW));
        assert!(rule_matches(&kind, &demo(1, 100 * MB + 1, 0, DemoStatus::Complete), NOW));
    }

    #[test]
    fn status_rules_match_only_their_status() {
        let d = |s| demo(1, 1, 0, s);
        assert!(rule_matches(&RuleKind::Outdated, &d(DemoStatus::Outdated), NOW));
        assert!(!rule_matches(&RuleKind::Outdated, &d(DemoStatus::Partial), NOW));
        assert!(rule_matches(&RuleKind::Partial, &d(DemoStatus::Partial), NOW));
        assert!(!rule_matches(&RuleKind::Partial, &d(DemoStatus::Complete), NOW));
    }

    #[test]
    fn a_replay_matching_any_enabled_rule_is_selected_with_every_rule_that_hit() {
        let demos = [
            demo(1, 1, 60, DemoStatus::Outdated),
            demo(2, 1, 1, DemoStatus::Complete),
            demo(3, 1, 60, DemoStatus::Complete),
        ];
        let rules = [rule(10, RuleKind::OlderThanDays { days: 30 }), rule(11, RuleKind::Outdated)];
        let got = select(&rules, &demos, &Pins::default(), NOW);
        assert_eq!(ids(&got), vec![1, 3]);
        assert_eq!(got[0].rule_ids, vec![10, 11]);
        assert_eq!(got[1].rule_ids, vec![10]);
        assert_eq!(got[0].file_name, "1.dem");
    }

    #[test]
    fn disabled_rules_do_nothing_and_no_rules_selects_nothing() {
        let demos = [demo(1, 1, 60, DemoStatus::Outdated)];
        let mut off = rule(1, RuleKind::Outdated);
        off.enabled = false;
        assert!(select(&[off], &demos, &Pins::default(), NOW).is_empty());
        assert!(select(&[], &demos, &Pins::default(), NOW).is_empty());
    }

    #[test]
    fn pinned_replays_are_never_selected() {
        let demos = [demo(1, 1, 60, DemoStatus::Outdated), demo(2, 1, 60, DemoStatus::Outdated)];
        let mut pins = Pins::default();
        pins.pin(1);
        let got = select(&[rule(1, RuleKind::Outdated)], &demos, &pins, NOW);
        assert_eq!(ids(&got), vec![2]);
    }

    #[test]
    fn zero_thresholds_and_duplicate_ids_are_rejected() {
        assert!(validate(&[rule(1, RuleKind::OlderThanDays { days: 0 })]).is_err());
        assert!(validate(&[rule(1, RuleKind::LargerThanMb { mb: 0 })]).is_err());
        assert!(validate(&[rule(1, RuleKind::Outdated), rule(1, RuleKind::Partial)]).is_err());
        assert!(validate(&[rule(1, RuleKind::OlderThanDays { days: 1 }), rule(2, RuleKind::Partial)]).is_ok());
    }

    #[test]
    fn rules_survive_a_save_and_load_in_the_frontend_shape() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("rules.json");
        let rules = vec![rule(1, RuleKind::OlderThanDays { days: 30 }), rule(2, RuleKind::Partial)];
        save_rules(&path, &rules).unwrap();
        assert_eq!(load_rules(&path), rules);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"kind\":\"olderThanDays\"") && text.contains("\"days\":30"), "{text}");
    }

    #[test]
    fn a_missing_or_corrupt_file_loads_as_no_rules() {
        let dir = temp_dir("corrupt");
        assert!(load_rules(&dir.join("nope.json")).is_empty());
        std::fs::write(dir.join("bad.json"), b"[{ nope").unwrap();
        assert!(load_rules(&dir.join("bad.json")).is_empty());
    }
}
