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

/// Points with `start <= t <= end`, thinned to at most `max_points` by keeping the worst ping of each bucket so
/// spikes survive. A bucket with no direct ping stays as a gap point.
pub fn points_in_range(points: &[HistoryPoint], start: u64, end: u64, max_points: usize) -> Vec<HistoryPoint> {
    let inside: Vec<&HistoryPoint> = points.iter().filter(|p| p.t >= start && p.t <= end).collect();
    let max_points = max_points.max(1);
    if inside.len() <= max_points {
        return inside.into_iter().cloned().collect();
    }
    let bucket = inside.len().div_ceil(max_points);
    inside
        .chunks(bucket)
        .map(|chunk| {
            chunk
                .iter()
                .filter(|p| p.raw.is_some())
                .max_by(|a, b| a.raw.partial_cmp(&b.raw).unwrap_or(std::cmp::Ordering::Equal))
                .map_or_else(|| (*chunk[0]).clone(), |p| (*p).clone())
        })
        .collect()
}

/// Reads the whole on-disk history, which outlives the monitor's in-memory window.
pub fn points_in_range_file(
    path: &Path,
    start: u64,
    end: u64,
    max_points: usize,
) -> std::io::Result<Vec<HistoryPoint>> {
    Ok(points_in_range(&HistoryStore::read_all(path)?, start, end, max_points))
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

    #[test]
    fn range_points_are_kept_whole_when_they_fit() {
        let points = [p(1, Some(5.0)), p(10, Some(20.0)), p(11, None), p(30, Some(90.0)), p(31, Some(1.0))];
        let got = points_in_range(&points, 10, 30, 10);
        assert_eq!(got.iter().map(|q| q.t).collect::<Vec<_>>(), vec![10, 11, 30]);
        assert_eq!(got[1].raw, None);
    }

    #[test]
    fn range_points_are_thinned_keeping_each_buckets_worst_ping() {
        let points: Vec<_> = (0..100u64).map(|t| p(t, Some(if t == 42 { 400.0 } else { 20.0 }))).collect();
        let got = points_in_range(&points, 0, 99, 10);
        assert!(got.len() <= 10);
        assert!(got.iter().any(|q| q.t == 42 && q.raw == Some(400.0)));
        assert!(got.windows(2).all(|w| w[0].t < w[1].t));
    }

    #[test]
    fn a_bucket_without_pings_stays_as_a_gap() {
        let points: Vec<_> = (0..20u64).map(|t| p(t, if t < 10 { None } else { Some(30.0) })).collect();
        let got = points_in_range(&points, 0, 19, 2);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].raw, None);
        assert_eq!(got[1].raw, Some(30.0));
    }

    #[test]
    fn range_points_read_from_a_file_and_a_missing_file_is_empty() {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-points", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("connection-history.jsonl");
        let lines: String = (1..=50u64)
            .map(|t| {
                format!(
                    "{{\"t\":{t},\"raw\":{t}.0}}
"
                )
            })
            .collect();
        fs::write(&path, lines).unwrap();
        assert_eq!(points_in_range_file(&path, 10, 19, 100).unwrap().len(), 10);
        assert!(points_in_range_file(&dir.join("missing.jsonl"), 0, 9, 10).unwrap().is_empty());
    }
}
