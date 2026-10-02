use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::PostGameMatch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "ProvisionalMatch")]
pub struct StoredMatch {
    #[serde(flatten)]
    #[ts(flatten)]
    pub game: PostGameMatch,
    pub captured_at: u64,
    /// Zero means the record predates account tracking and belongs to nobody.
    #[serde(default)]
    pub account_id: u32,
}

pub fn for_account(matches: &[StoredMatch], account_id: u32) -> Vec<StoredMatch> {
    matches.iter().filter(|m| account_id != 0 && m.account_id == account_id).cloned().collect()
}

pub fn upsert(matches: &mut Vec<StoredMatch>, account_id: u32, game: PostGameMatch, now: u64) -> StoredMatch {
    let stored = StoredMatch { game, captured_at: now, account_id };
    match matches.iter_mut().find(|m| m.account_id == account_id && m.game.match_id == stored.game.match_id) {
        Some(slot) => *slot = stored.clone(),
        None => matches.push(stored.clone()),
    }
    stored
}

/// Drops `account_id`'s matches the API now has, matches older than `max_age_secs`, and matches stored
/// without an account. Returns how many were removed.
pub fn reconcile(
    matches: &mut Vec<StoredMatch>,
    account_id: u32,
    api_ids: &[u64],
    now: u64,
    max_age_secs: u64,
) -> usize {
    let before = matches.len();
    matches.retain(|m| {
        m.account_id != 0
            && !(m.account_id == account_id && api_ids.contains(&m.game.match_id))
            && now.saturating_sub(m.captured_at) <= max_age_secs
    });
    before - matches.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Outcome;

    const ME: u32 = 7;
    const OTHER: u32 = 8;

    fn game(match_id: u64, kills: u32) -> PostGameMatch {
        PostGameMatch {
            match_id,
            hero_id: 1,
            start_time: 100,
            match_mode: 4,
            game_mode: 1,
            outcome: Outcome::Win,
            kills,
            deaths: 0,
            assists: 0,
            net_worth: 0,
            duration_s: 0,
            rank_badge: 0,
            rank_delta: None,
            calibration: false,
            demotion_protected: false,
        }
    }

    fn at(match_id: u64, captured_at: u64) -> StoredMatch {
        StoredMatch { game: game(match_id, 0), captured_at, account_id: ME }
    }

    #[test]
    fn upsert_appends_a_new_match() {
        let mut v = vec![at(1, 10)];
        let stored = upsert(&mut v, ME, game(2, 5), 20);
        assert_eq!(v.len(), 2);
        assert_eq!(stored, StoredMatch { game: game(2, 5), captured_at: 20, account_id: ME });
        assert_eq!(v[1], stored);
    }

    #[test]
    fn upsert_replaces_the_same_match_id_in_place() {
        let mut v = vec![at(1, 10), at(2, 10)];
        upsert(&mut v, ME, game(1, 9), 30);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], StoredMatch { game: game(1, 9), captured_at: 30, account_id: ME });
        assert_eq!(v[1].game.match_id, 2);
    }

    #[test]
    fn reconcile_drops_matches_the_api_has() {
        let mut v = vec![at(1, 100), at(2, 100), at(3, 100)];
        assert_eq!(reconcile(&mut v, ME, &[1, 3, 99], 110, 1000), 2);
        assert_eq!(v.iter().map(|m| m.game.match_id).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn reconcile_drops_matches_older_than_the_max_age() {
        let mut v = vec![at(1, 0), at(2, 500), at(3, 1000)];
        assert_eq!(reconcile(&mut v, ME, &[], 1500, 1000), 1);
        assert_eq!(v.iter().map(|m| m.game.match_id).collect::<Vec<_>>(), vec![2, 3]);
    }

    #[test]
    fn reconcile_counts_a_match_once_when_both_rules_apply() {
        let mut v = vec![at(1, 0)];
        assert_eq!(reconcile(&mut v, ME, &[1], 5000, 1000), 1);
        assert!(v.is_empty());
    }

    #[test]
    fn serialises_flat_camel_case_with_captured_at() {
        let json = serde_json::to_value(at(7, 42)).unwrap();
        assert_eq!(json["matchId"], 7);
        assert_eq!(json["capturedAt"], 42);
        let back: StoredMatch = serde_json::from_value(json).unwrap();
        assert_eq!(back, at(7, 42));
    }

    fn owned(match_id: u64, account_id: u32) -> StoredMatch {
        StoredMatch { account_id, ..at(match_id, 100) }
    }

    #[test]
    fn upsert_keeps_the_same_match_id_of_another_account_separate() {
        let mut v = vec![owned(1, OTHER)];
        upsert(&mut v, ME, game(1, 2), 30);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn for_account_returns_only_that_accounts_matches() {
        let v = vec![owned(1, ME), owned(2, OTHER), owned(3, ME)];
        let ids: Vec<_> = for_account(&v, ME).iter().map(|m| m.game.match_id).collect();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn for_account_never_returns_records_without_an_account() {
        let v = vec![owned(1, 0)];
        assert!(for_account(&v, 0).is_empty());
    }

    #[test]
    fn reconcile_only_applies_the_api_rule_to_the_given_account() {
        let mut v = vec![owned(1, ME), owned(2, OTHER)];
        assert_eq!(reconcile(&mut v, ME, &[1, 2], 110, 1000), 1);
        assert_eq!(v[0].game.match_id, 2);
    }

    #[test]
    fn reconcile_ages_out_every_account_and_drops_unowned_records() {
        let mut v = vec![owned(1, OTHER), owned(2, 0), owned(3, ME)];
        v[0].captured_at = 0;
        assert_eq!(reconcile(&mut v, ME, &[], 1050, 1000), 2);
        assert_eq!(v[0].game.match_id, 3);
    }

    #[test]
    fn a_record_stored_without_an_account_deserialises_as_unowned() {
        let mut json = serde_json::to_value(at(7, 42)).unwrap();
        json.as_object_mut().unwrap().remove("accountId");
        let back: StoredMatch = serde_json::from_value(json).unwrap();
        assert_eq!(back.account_id, 0);
    }
}
