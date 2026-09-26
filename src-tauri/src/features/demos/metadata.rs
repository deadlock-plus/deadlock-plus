use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use ts_rs::TS;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::features::versioned::{self, Migration};

const MIGRATIONS: &[Migration] = &[];

const API: &str = "https://api.deadlock-api.com/v1/matches";
/// Matches the API has not processed yet return 404 now and may exist later.
const MISSING_RETRY_SECS: u64 = 6 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSummary {
    pub account_id: u32,
    pub hero_id: u32,
    pub team: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

/// The API answer is about 1 MB per match; only these fields are kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MatchSummary {
    pub match_id: u64,
    pub start_time: u64,
    pub duration_s: u32,
    pub winning_team: u32,
    pub players: Vec<PlayerSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum CacheEntry {
    Ok { summary: MatchSummary },
    Missing { fetched_at: u64 },
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum MetaResult {
    Ok { summary: MatchSummary },
    Missing,
    Error { message: String },
}

fn uint(v: &Value, key: &str) -> Option<u64> {
    v.get(key)?.as_u64()
}

fn summarize_player(p: &Value) -> Option<PlayerSummary> {
    Some(PlayerSummary {
        account_id: u32::try_from(uint(p, "account_id")?).ok()?,
        hero_id: u32::try_from(uint(p, "hero_id")?).ok()?,
        team: u32::try_from(uint(p, "team")?).ok()?,
        kills: u32::try_from(uint(p, "kills")?).ok()?,
        deaths: u32::try_from(uint(p, "deaths")?).ok()?,
        assists: u32::try_from(uint(p, "assists")?).ok()?,
    })
}

pub fn summarize(body: &Value) -> Option<MatchSummary> {
    let info = body.get("match_info")?;
    Some(MatchSummary {
        match_id: uint(info, "match_id")?,
        start_time: uint(info, "start_time")?,
        duration_s: u32::try_from(uint(info, "duration_s")?).ok()?,
        winning_team: u32::try_from(uint(info, "winning_team")?).ok()?,
        players: info.get("players")?.as_array()?.iter().filter_map(summarize_player).collect(),
    })
}

fn should_refetch(entry: Option<&CacheEntry>, now: u64) -> bool {
    match entry {
        None => true,
        Some(CacheEntry::Ok { .. }) => false,
        Some(CacheEntry::Missing { fetched_at }) => now.saturating_sub(*fetched_at) >= MISSING_RETRY_SECS,
    }
}

struct Loaded {
    path: PathBuf,
    entries: HashMap<u64, CacheEntry>,
}

#[derive(Default)]
pub struct DemoMetaCache(Mutex<Option<Loaded>>);

impl DemoMetaCache {
    fn with<R>(&self, dir: &std::path::Path, f: impl FnOnce(&mut Loaded) -> R) -> R {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let loaded = guard.get_or_insert_with(|| {
            let path = dir.join("demo-metadata.json");
            let entries = versioned::read(&path, MIGRATIONS)
                .unwrap_or_else(|e| {
                    log::warn!("could not read the replay metadata cache, starting empty: {e}");
                    None
                })
                .unwrap_or_default();
            Loaded { path, entries }
        });
        f(loaded)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

async fn fetch(match_id: u64) -> MetaResult {
    let error = |m: String| MetaResult::Error { message: m };
    let res = match reqwest::get(format!("{API}/{match_id}/metadata")).await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("metadata request for match {match_id} failed: {e}");
            return error(e.to_string());
        }
    };
    match res.status().as_u16() {
        404 => return MetaResult::Missing,
        200 => {}
        429 => {
            log::warn!("metadata request for match {match_id} was rate limited");
            return error("The Deadlock API is rate limiting requests. Try again shortly.".into());
        }
        code => {
            log::warn!("metadata request for match {match_id} returned {code}");
            return error(format!("The Deadlock API returned {code}."));
        }
    }
    match res.json::<Value>().await.map(|v| summarize(&v)) {
        Ok(Some(summary)) => MetaResult::Ok { summary },
        Ok(None) => error("The Deadlock API returned an unexpected match format.".into()),
        Err(e) => error(e.to_string()),
    }
}

pub mod commands {
    use super::{fetch, now_secs, should_refetch, CacheEntry, DemoMetaCache, MetaResult};
    use tauri::{Manager, State};

    #[tauri::command]
    pub async fn demo_metadata(
        app: tauri::AppHandle,
        cache: State<'_, DemoMetaCache>,
        match_id: u64,
    ) -> Result<MetaResult, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let now = now_secs();
        let cached = cache.with(&dir, |c| {
            let entry = c.entries.get(&match_id);
            if should_refetch(entry, now) {
                None
            } else {
                Some(match entry {
                    Some(CacheEntry::Ok { summary }) => MetaResult::Ok { summary: summary.clone() },
                    _ => MetaResult::Missing,
                })
            }
        });
        if let Some(hit) = cached {
            return Ok(hit);
        }
        let result = fetch(match_id).await;
        let entry = match &result {
            MetaResult::Ok { summary } => Some(CacheEntry::Ok { summary: summary.clone() }),
            MetaResult::Missing => Some(CacheEntry::Missing { fetched_at: now }),
            MetaResult::Error { .. } => None,
        };
        if let Some(entry) = entry {
            cache.with(&dir, |c| {
                c.entries.insert(match_id, entry);
                if let Err(e) = crate::features::versioned::write(&c.path, super::MIGRATIONS, &c.entries) {
                    log::warn!("could not write the replay metadata cache: {e}");
                }
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn player(id: u32, hero: u32, team: u32) -> Value {
        json!({ "account_id": id, "hero_id": hero, "team": team, "kills": 6, "deaths": 8, "assists": 20, "items": [1, 2, 3] })
    }

    fn body() -> Value {
        json!({ "match_info": {
            "match_id": 106837748u64, "start_time": 1789947499u64, "duration_s": 2070, "winning_team": 1,
            "players": [player(395693146, 65, 1), player(251353065, 19, 0)],
            "damage_matrix": { "huge": [1, 2, 3] },
        } })
    }

    #[test]
    fn keeps_only_the_summary_fields() {
        let s = summarize(&body()).unwrap();
        assert_eq!(s.match_id, 106837748);
        assert_eq!(s.start_time, 1789947499);
        assert_eq!(s.duration_s, 2070);
        assert_eq!(s.winning_team, 1);
        assert_eq!(
            s.players[0],
            PlayerSummary { account_id: 395693146, hero_id: 65, team: 1, kills: 6, deaths: 8, assists: 20 }
        );
        assert_eq!(s.players.len(), 2);
    }

    #[test]
    fn a_player_missing_a_field_is_skipped_not_fatal() {
        let mut b = body();
        b["match_info"]["players"][0].as_object_mut().unwrap().remove("hero_id");
        assert_eq!(summarize(&b).unwrap().players.len(), 1);
    }

    #[test]
    fn a_body_without_the_core_fields_is_rejected() {
        assert_eq!(summarize(&json!({})), None);
        assert_eq!(summarize(&json!({ "match_info": {} })), None);
        let mut b = body();
        b["match_info"].as_object_mut().unwrap().remove("duration_s");
        assert_eq!(summarize(&b), None);
    }

    #[test]
    fn found_matches_are_never_refetched_and_missing_ones_after_a_delay() {
        let ok = CacheEntry::Ok { summary: summarize(&body()).unwrap() };
        assert!(should_refetch(None, 0));
        assert!(!should_refetch(Some(&ok), u64::MAX));
        let missing = CacheEntry::Missing { fetched_at: 1_000 };
        assert!(!should_refetch(Some(&missing), 1_000 + MISSING_RETRY_SECS - 1));
        assert!(should_refetch(Some(&missing), 1_000 + MISSING_RETRY_SECS));
        assert!(!should_refetch(Some(&missing), 500));
    }

    #[test]
    fn cache_entries_round_trip_through_json() {
        let mut m = HashMap::new();
        m.insert(1u64, CacheEntry::Ok { summary: summarize(&body()).unwrap() });
        m.insert(2u64, CacheEntry::Missing { fetched_at: 9 });
        let back: HashMap<u64, CacheEntry> = serde_json::from_slice(&serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(matches!(back[&1], CacheEntry::Ok { .. }));
        assert!(matches!(back[&2], CacheEntry::Missing { fetched_at: 9 }));
    }
}
