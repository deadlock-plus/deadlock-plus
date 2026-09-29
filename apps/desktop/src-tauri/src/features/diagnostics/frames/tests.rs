use super::*;

const TICKS_PER_SEC: u64 = 10_000_000;
const TICKS_PER_MS: u64 = TICKS_PER_SEC / 1000;

fn stamps(frametimes_ms: &[f64]) -> Vec<u64> {
    let mut t = 1_000_000u64;
    let mut out = vec![t];
    for ms in frametimes_ms {
        t += (ms * TICKS_PER_MS as f64).round() as u64;
        out.push(t);
    }
    out
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

#[test]
fn empty_and_single_timestamp_yield_no_frames() {
    for input in [vec![], vec![42]] {
        let stats = FrameStats::from_timestamps(&input, TICKS_PER_SEC);
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.max_ms, 0.0);
        assert!(stats.spikes.is_empty());
    }
}

#[test]
fn steady_cadence_has_flat_percentiles_and_no_spikes() {
    let stats = FrameStats::from_timestamps(&stamps(&[16.6; 100]), TICKS_PER_SEC);
    assert_eq!(stats.frame_count, 100);
    assert!(near(stats.duration_ms, 1660.0));
    assert!(near(stats.avg_ms, 16.6));
    assert!(near(stats.median_ms, 16.6));
    assert!(near(stats.p95_ms, 16.6));
    assert!(near(stats.p99_ms, 16.6));
    assert!(near(stats.max_ms, 16.6));
    assert!(near(stats.low_1pct_fps, 1000.0 / 16.6));
    assert!(stats.spikes.is_empty());
}

#[test]
fn one_stall_shows_in_tail_percentiles_and_spike_list() {
    let mut frames = vec![16.0; 100];
    frames.insert(40, 60.0);
    let stats = FrameStats::from_timestamps(&stamps(&frames), TICKS_PER_SEC);
    assert_eq!(stats.frame_count, 101);
    assert!(near(stats.median_ms, 16.0));
    assert!(near(stats.p99_ms, 16.0));
    assert!(near(stats.p999_ms, 60.0));
    assert!(near(stats.max_ms, 60.0));
    assert!(near(stats.low_01pct_fps, 1000.0 / 60.0));
    assert_eq!(stats.spikes.len(), 1);
    assert!(near(stats.spikes[0].frametime_ms, 60.0));
    assert!(near(stats.spikes[0].at_ms, 40.0 * 16.0));
}

#[test]
fn unsorted_input_matches_sorted_input() {
    let sorted = stamps(&[16.0, 16.0, 70.0, 16.0, 16.0]);
    let mut shuffled = sorted.clone();
    shuffled.reverse();
    shuffled.swap(1, 3);
    assert_eq!(
        FrameStats::from_timestamps(&shuffled, TICKS_PER_SEC),
        FrameStats::from_timestamps(&sorted, TICKS_PER_SEC)
    );
}

#[test]
fn slow_frames_under_the_absolute_floor_are_not_spikes() {
    let mut frames = vec![5.0; 50];
    frames.push(14.0);
    let stats = FrameStats::from_timestamps(&stamps(&frames), TICKS_PER_SEC);
    assert!(stats.spikes.is_empty());
}

#[test]
fn duplicate_timestamps_count_as_zero_length_frames() {
    let stats = FrameStats::from_timestamps(&[100, 100, 100 + 166_000], TICKS_PER_SEC);
    assert_eq!(stats.frame_count, 2);
    assert!(near(stats.median_ms, 8.3));
}

#[test]
fn zero_tick_rate_does_not_panic() {
    let stats = FrameStats::from_timestamps(&[1, 2, 3], 0);
    assert_eq!(stats.frame_count, 0);
}

#[test]
fn recent_frametimes_returns_the_newest_deltas_in_order() {
    let ts = stamps(&[10.0, 20.0, 30.0, 40.0]);
    let recent = recent_frametimes_ms(&ts, 2, TICKS_PER_SEC);
    assert_eq!(recent.len(), 2);
    assert!(near(recent[0] as f64, 30.0) && near(recent[1] as f64, 40.0));
    assert!(recent_frametimes_ms(&ts[..1], 5, TICKS_PER_SEC).is_empty());
    assert!(recent_frametimes_ms(&ts, 5, 0).is_empty());
}

#[test]
fn split_focused_drops_unfocused_frames_and_breaks_the_segment() {
    let ts: Vec<u64> = (0..=10).collect();
    let segments = split_focused(&ts, &[(4, 6)]);
    assert_eq!(segments, vec![vec![0, 1, 2, 3], vec![7, 8, 9, 10]]);
}

#[test]
fn split_focused_breaks_at_an_interval_that_falls_between_frames() {
    let segments = split_focused(&[0, 10, 20, 30], &[(12, 18)]);
    assert_eq!(segments, vec![vec![0, 10], vec![20, 30]]);
}

#[test]
fn split_focused_without_intervals_keeps_one_sorted_segment() {
    assert_eq!(split_focused(&[3, 1, 2], &[]), vec![vec![1, 2, 3]]);
    assert!(split_focused(&[], &[(0, 5)]).is_empty());
}

#[test]
fn a_background_gap_does_not_produce_a_spike() {
    let first = stamps(&[16.0; 50]);
    let resume = first.last().unwrap() + 5 * TICKS_PER_SEC;
    let second: Vec<u64> = (0..=50).map(|i| resume + i * 16 * TICKS_PER_MS).collect();
    let stats = FrameStats::from_segments(&[first, second], TICKS_PER_SEC);
    assert_eq!(stats.frame_count, 100);
    assert!(stats.spikes.is_empty());
    assert!(near(stats.max_ms, 16.0));
    assert!(near(stats.duration_ms, 1600.0));
}
