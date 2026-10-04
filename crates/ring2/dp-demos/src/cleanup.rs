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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleError {
    DuplicateId,
    ZeroDays,
    ZeroSize,
}

/// A zero threshold would match every replay, so it is rejected instead of clamped.
pub fn validate(rules: &[Rule]) -> Result<(), RuleError> {
    let mut seen = std::collections::HashSet::new();
    for rule in rules {
        if !seen.insert(rule.id) {
            return Err(RuleError::DuplicateId);
        }
        match rule.kind {
            RuleKind::OlderThanDays { days: 0 } => return Err(RuleError::ZeroDays),
            RuleKind::LargerThanMb { mb: 0 } => return Err(RuleError::ZeroSize),
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
        assert_eq!(validate(&[rule(1, RuleKind::OlderThanDays { days: 0 })]), Err(RuleError::ZeroDays));
        assert_eq!(validate(&[rule(1, RuleKind::LargerThanMb { mb: 0 })]), Err(RuleError::ZeroSize));
        assert_eq!(validate(&[rule(1, RuleKind::Outdated), rule(1, RuleKind::Partial)]), Err(RuleError::DuplicateId));
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
