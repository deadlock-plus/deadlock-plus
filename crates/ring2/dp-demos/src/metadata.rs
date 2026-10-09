use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ts_rs::TS;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use dp_versioned::{self, Migration};

const MIGRATIONS: &[Migration] = &[];

const API: &str = "https://api.deadlock-api.com/v1/matches";
/// Matches the API has not processed yet return 404 now and may exist later.
const WRITE_DEBOUNCE: Duration = Duration::from_secs(2);
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

pub struct DemoMetaCache {
    state: Arc<Mutex<Option<Loaded>>>,
    write_pending: Arc<AtomicBool>,
    debounce: Duration,
}

impl Default for DemoMetaCache {
    fn default() -> Self {
        Self::with_debounce(WRITE_DEBOUNCE)
    }
}

impl DemoMetaCache {
    fn with_debounce(debounce: Duration) -> Self {
        Self { state: Arc::default(), write_pending: Arc::default(), debounce }
    }

    fn with<R>(&self, dir: &std::path::Path, f: impl FnOnce(&mut Loaded) -> R) -> R {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let loaded = guard.get_or_insert_with(|| {
            let path = dir.join("demo-metadata.json");
            let entries = dp_versioned::read(&path, MIGRATIONS)
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

impl DemoMetaCache {
    /// Records `entry` and writes the file once, shortly after, on its own thread, so a burst of
    /// lookups costs one write and never blocks the async runtime.
    fn store(&self, dir: &std::path::Path, match_id: u64, entry: CacheEntry) {
        self.with(dir, |c| {
            c.entries.insert(match_id, entry);
        });
        if self.write_pending.swap(true, Ordering::SeqCst) {
            return;
        }
        let (state, pending, debounce) = (self.state.clone(), self.write_pending.clone(), self.debounce);
        let spawned = thread::Builder::new().name("demo-meta-write".into()).spawn(move || {
            thread::sleep(debounce);
            pending.store(false, Ordering::SeqCst);
            let snapshot =
                state.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|c| (c.path.clone(), c.entries.clone()));
            if let Some((path, entries)) = snapshot {
                if let Err(e) = dp_versioned::write(&path, MIGRATIONS, &entries) {
                    log::warn!("could not write the replay metadata cache: {e}");
                }
            }
        });
        if let Err(e) = spawned {
            self.write_pending.store(false, Ordering::SeqCst);
            log::warn!("could not start the replay metadata cache writer: {e}");
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

async fn fetch(client: &reqwest::Client, match_id: u64) -> MetaResult {
    let error = |m: String| MetaResult::Error { message: m };
    let res = match client.get(format!("{API}/{match_id}/metadata")).send().await {
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

pub async fn lookup(
    cache: &DemoMetaCache,
    client: &reqwest::Client,
    dir: &std::path::Path,
    match_id: u64,
) -> MetaResult {
    let now = now_secs();
    let cached = cache.with(dir, |c| {
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
        return hit;
    }
    let result = fetch(client, match_id).await;
    let entry = match &result {
        MetaResult::Ok { summary } => Some(CacheEntry::Ok { summary: summary.clone() }),
        MetaResult::Missing => Some(CacheEntry::Missing { fetched_at: now }),
        MetaResult::Error { .. } => None,
    };
    if let Some(entry) = entry {
        cache.store(dir, match_id, entry);
    }
    result
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
    fn stored_lookups_are_written_after_the_debounce_not_immediately() {
        let dir = std::env::temp_dir().join(format!("dp-demos-meta-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("demo-metadata.json");
        let cache = DemoMetaCache::with_debounce(Duration::from_millis(300));
        for id in 1..=3u64 {
            cache.store(&dir, id, CacheEntry::Missing { fetched_at: id });
        }
        assert!(!file.exists());
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !file.exists() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        let fresh = DemoMetaCache::default();
        let count = fresh.with(&dir, |c| c.entries.len());
        assert_eq!(count, 3);
        let _ = std::fs::remove_dir_all(&dir);
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
