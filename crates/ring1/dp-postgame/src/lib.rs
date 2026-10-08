use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod capture;
mod map;
mod store;
mod watch;
mod worker;
pub use capture::{from_event, Capture};
pub use map::to_match;
pub use store::{for_account, reconcile, upsert, StoredMatch};
pub use watch::{plan, Plan};
pub use worker::{Done, Stop, Worker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Win,
    Loss,
    Unscored,
}

/// One finished match as the client held it in memory. Field names match the frontend `Match`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PostGameMatch {
    pub match_id: u64,
    pub hero_id: u32,
    pub start_time: u64,
    pub match_mode: u32,
    pub game_mode: u32,
    pub outcome: Outcome,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub net_worth: u32,
    pub duration_s: u32,
    pub rank_badge: u32,
    pub rank_delta: Option<i32>,
    pub calibration: bool,
    pub demotion_protected: bool,
}
