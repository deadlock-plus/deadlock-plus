use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::match_store::{MatchPing, PingSource};

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

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, rename = "NetworkSnapshot")]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub monitoring: bool,
    /// Live monitoring needs administrator rights the user has not granted yet.
    pub needs_permission: bool,
    pub trace_error: Option<String>,
    pub game_running: bool,
    pub relay: Option<RelayInfo>,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPoint {
    pub t: u64,
    pub raw: Option<f32>,
}

/// Where a match's ping curve was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum PingSeriesSource {
    /// The game's own connection to the server.
    Engine,
    /// A round trip to the relay.
    Icmp,
}

/// One match's recorded ping curve, thinned for drawing.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MatchPingSeries {
    pub source: PingSeriesSource,
    /// The recording began after the match had started.
    pub partial: bool,
    pub points: Vec<HistoryPoint>,
}

impl MatchPingSeries {
    pub fn from_ping(ping: &MatchPing, max_points: usize) -> Self {
        let source = match ping.header.source {
            PingSource::Engine => PingSeriesSource::Engine,
            PingSource::Icmp => PingSeriesSource::Icmp,
        };
        Self { source, partial: ping.header.partial, points: ping.downsample(max_points) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_store::{MatchHeader, MatchPing, MatchSample, PingSource};

    fn ping(source: PingSource, partial: bool, n: u32) -> MatchPing {
        MatchPing {
            header: MatchHeader { version: 1, source, partial, start_ms: 10_000 },
            points: (0..n).map(|i| MatchSample { offset_ms: i * 1000, ping_ms: Some(20.0), engine: None }).collect(),
        }
    }

    #[test]
    fn a_series_carries_the_header_and_downsampled_points() {
        let series = MatchPingSeries::from_ping(&ping(PingSource::Engine, true, 1000), 100);
        assert_eq!(series.source, PingSeriesSource::Engine);
        assert!(series.partial);
        assert!(series.points.len() <= 100 && !series.points.is_empty());
        assert!(series.points[0].t >= 10_000);
    }

    #[test]
    fn the_series_serialises_with_a_lowercase_source() {
        let series = MatchPingSeries::from_ping(&ping(PingSource::Icmp, false, 2), 10);
        let json = serde_json::to_value(&series).unwrap();
        assert_eq!(json["source"], "icmp");
        assert_eq!(json["partial"], false);
        assert_eq!(json["points"].as_array().unwrap().len(), 2);
    }
}
