use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use crate::history_store::HistoryStore;
use crate::types::HistoryPoint;

/// Direct-to-relay ping over a span of history. Loss is not recorded per point, so it is not summarised.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PingSummary {
    pub avg: f32,
    pub worst: f32,
    pub samples: u32,
}

/// Points with `start <= t <= end` that carry a direct ping. `None` when there are none.
pub fn summarize(points: &[HistoryPoint], start: u64, end: u64) -> Option<PingSummary> {
    let mut sum = 0.0f64;
    let mut worst = f32::MIN;
    let mut samples = 0u32;
    for raw in points.iter().filter(|p| p.t >= start && p.t <= end).filter_map(|p| p.raw) {
        sum += f64::from(raw);
        worst = worst.max(raw);
        samples += 1;
    }
    (samples > 0).then(|| PingSummary { avg: (sum / f64::from(samples)) as f32, worst, samples })
}

/// Reads the whole on-disk history, which outlives the monitor's in-memory window.
pub fn summarize_file(path: &Path, start: u64, end: u64) -> std::io::Result<Option<PingSummary>> {
    Ok(summarize(&HistoryStore::read_all(path)?, start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn p(t: u64, raw: Option<f32>) -> HistoryPoint {
        HistoryPoint { t, raw }
    }

    #[test]
    fn averages_and_finds_the_worst_inside_the_span_only() {
        let points = [p(1, Some(500.0)), p(10, Some(20.0)), p(20, Some(40.0)), p(30, Some(90.0)), p(31, Some(500.0))];
        let s = summarize(&points, 10, 30).unwrap();
        assert_eq!(s, PingSummary { avg: 50.0, worst: 90.0, samples: 3 });
    }

    #[test]
    fn dropped_samples_are_skipped_not_counted_as_zero() {
        let points = [p(1, Some(30.0)), p(2, None), p(3, Some(50.0))];
        let s = summarize(&points, 0, 10).unwrap();
        assert_eq!((s.avg, s.samples), (40.0, 2));
    }

    #[test]
    fn a_span_without_pings_has_no_summary() {
        assert_eq!(summarize(&[p(1, None)], 0, 10), None);
        assert_eq!(summarize(&[p(50, Some(10.0))], 0, 10), None);
        assert_eq!(summarize(&[], 0, 10), None);
    }

    #[test]
    fn reads_more_points_than_the_store_keeps_in_memory() {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-range", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("connection-history.jsonl");
        let lines: String = (1..=2000u64).map(|t| format!("{{\"t\":{t},\"raw\":{t}.0,\"exit\":null}}\n")).collect();
        fs::write(&path, lines).unwrap();

        let s = summarize_file(&path, 1, 2000).unwrap().unwrap();
        assert_eq!((s.samples, s.worst), (2000, 2000.0));
        assert!(summarize_file(&dir.join("missing.jsonl"), 0, 1).unwrap().is_none());
    }
}
