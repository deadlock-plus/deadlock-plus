use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PingStats {
    pub current: Option<f32>,
    pub avg: Option<f32>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub jitter: Option<f32>,
    pub loss_pct: f32,
    pub samples: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RelayInfo {
    pub ip: String,
    pub port: u16,
    pub pop_code: Option<String>,
    pub description: Option<String>,
    pub country_code: Option<String>,
    pub pps_in: f32,
    pub pps_out: f32,
    /// Longest gap between inbound packets in the last second.
    pub max_gap_ms: f32,
    pub ping: PingStats,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EndpointInfo {
    pub ip: String,
    pub port: u16,
    pub pps: f32,
    pub is_exit: bool,
    pub ping: PingStats,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, rename = "NetworkSnapshot")]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub monitoring: bool,
    /// Live monitoring needs administrator rights the user has not granted yet.
    pub needs_permission: bool,
    pub trace_error: Option<String>,
    pub game_running: bool,
    pub exitlag_running: bool,
    pub relay: Option<RelayInfo>,
    pub exitlag_endpoints: Vec<EndpointInfo>,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPoint {
    pub t: u64,
    pub raw: Option<f32>,
    pub exit: Option<f32>,
}
