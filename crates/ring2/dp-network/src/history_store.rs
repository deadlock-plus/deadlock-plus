use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::types::HistoryPoint;

/// Append-only JSON Lines log of ping history. One point per line so a crash mid-write
/// only ever damages the last line, which the loader skips.
pub struct HistoryStore {
    path: PathBuf,
    keep: usize,
    lines: usize,
    file: File,
    compacting: bool,
    pending: Vec<HistoryPoint>,
}

/// A trim of the log that runs off the appending thread. Holds no reference to the store, so it can run
/// without any lock while the store keeps accepting points.
pub struct Compaction {
    path: PathBuf,
    keep: usize,
}

impl Compaction {
    /// Rewrites the file trimmed to the newest `keep` points and returns how many lines it now holds.
    pub fn run(self) -> std::io::Result<usize> {
        compact(&self.path, self.keep).map(|points| points.len())
    }
}

fn load(path: &Path, keep: usize) -> std::io::Result<Vec<HistoryPoint>> {
    let mut points: Vec<HistoryPoint> = match fs::read_to_string(path) {
        Ok(text) => text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e),
    };
    if points.len() > keep {
        points.drain(..points.len() - keep);
    }
    Ok(points)
}

fn compact(path: &Path, keep: usize) -> std::io::Result<Vec<HistoryPoint>> {
    let points = load(path, keep)?;
    let mut compacted = Vec::new();
    for p in &points {
        writeln!(compacted, "{}", serde_json::to_string(p)?)?;
    }
    dp_atomic::write_atomic(path, &compacted)?;
    Ok(points)
}

impl HistoryStore {
    /// Every readable point in the log, without trimming or rewriting it. A missing file is empty.
    pub fn read_all(path: &Path) -> std::io::Result<Vec<HistoryPoint>> {
        load(path, usize::MAX)
    }

    /// Loads the newest `keep` points, rewrites the file trimmed to them (dropping corrupt
    /// lines), and opens it for appending.
    pub fn open(path: &Path, keep: usize) -> std::io::Result<(Self, Vec<HistoryPoint>)> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let points = compact(path, keep)?;
        let file = OpenOptions::new().append(true).open(path)?;
        let store =
            Self { path: path.to_path_buf(), keep, lines: points.len(), file, compacting: false, pending: Vec::new() };
        Ok((store, points))
    }

    /// While a compaction is in flight the point waits in memory: the rewrite replaces the file, so a line
    /// appended to it meanwhile would be lost.
    pub fn append(&mut self, point: &HistoryPoint) -> std::io::Result<()> {
        if self.compacting {
            self.pending.push(point.clone());
            return Ok(());
        }
        writeln!(self.file, "{}", serde_json::to_string(point)?)?;
        self.lines += 1;
        Ok(())
    }

    /// Once the log holds twice `keep` lines it is due a trim, so a session that never restarts still has a
    /// bounded log. Hand the result to `Compaction::run`, then pass that outcome to `finish_compaction`.
    pub fn take_compaction(&mut self) -> Option<Compaction> {
        if self.compacting || self.lines <= self.keep.saturating_mul(2) {
            return None;
        }
        self.compacting = true;
        Some(Compaction { path: self.path.clone(), keep: self.keep })
    }

    /// Reopens the append handle on the (possibly replaced) file and writes what arrived meanwhile.
    /// A failed compaction leaves the file as it was, so the points are simply appended to it.
    pub fn finish_compaction(&mut self, outcome: std::io::Result<usize>) -> std::io::Result<()> {
        self.compacting = false;
        if let Ok(lines) = outcome {
            self.lines = lines;
        }
        self.file = OpenOptions::new().append(true).open(&self.path)?;
        for point in std::mem::take(&mut self.pending) {
            self.append(&point)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("connection-history.jsonl")
    }

    fn point(t: u64) -> HistoryPoint {
        HistoryPoint { t, raw: Some(t as f32) }
    }

    #[test]
    fn missing_file_yields_no_points_and_creates_the_file() {
        let path = temp_path("missing");
        let (_, points) = HistoryStore::open(&path, 10).unwrap();
        assert!(points.is_empty());
        assert!(path.exists());
    }

    #[test]
    fn appended_points_are_loaded_by_the_next_open() {
        let path = temp_path("roundtrip");
        let (mut store, _) = HistoryStore::open(&path, 10).unwrap();
        store.append(&point(1)).unwrap();
        store.append(&point(2)).unwrap();
        drop(store);

        let (_, points) = HistoryStore::open(&path, 10).unwrap();
        assert_eq!(points.iter().map(|p| p.t).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(points[1].raw, Some(2.0));
    }

    #[test]
    fn open_keeps_only_the_newest_points_and_trims_the_file() {
        let path = temp_path("trim");
        let (mut store, _) = HistoryStore::open(&path, 10).unwrap();
        for t in 1..=5 {
            store.append(&point(t)).unwrap();
        }
        drop(store);

        let (_, points) = HistoryStore::open(&path, 3).unwrap();
        assert_eq!(points.iter().map(|p| p.t).collect::<Vec<_>>(), vec![3, 4, 5]);
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 3);
    }

    #[test]
    fn corrupt_lines_are_skipped_and_dropped_from_the_file() {
        let path = temp_path("corrupt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "{\"t\":1,\"raw\":1.0,\"exit\":null}\nnot json\n{\"t\":2,\"raw\":null,\"exit\":null}\n{\"t\":3,\"ra",
        )
        .unwrap();

        let (_, points) = HistoryStore::open(&path, 10).unwrap();
        assert_eq!(points.iter().map(|p| p.t).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 2);
    }

    #[test]
    fn a_long_session_keeps_the_file_bounded_and_the_newest_points() {
        let path = temp_path("bounded");
        let (mut store, _) = HistoryStore::open(&path, 3).unwrap();
        for t in 1..=20 {
            store.append(&point(t)).unwrap();
            if let Some(job) = store.take_compaction() {
                store.finish_compaction(job.run()).unwrap();
            }
            assert!(fs::read_to_string(&path).unwrap().lines().count() <= 7);
        }
        drop(store);

        let (_, points) = HistoryStore::open(&path, 3).unwrap();
        assert_eq!(points.iter().map(|p| p.t).collect::<Vec<_>>(), vec![18, 19, 20]);
    }

    #[test]
    fn appending_after_a_reopen_continues_the_log() {
        let path = temp_path("reopen");
        let (mut store, _) = HistoryStore::open(&path, 10).unwrap();
        store.append(&point(1)).unwrap();
        drop(store);
        let (mut store, _) = HistoryStore::open(&path, 10).unwrap();
        store.append(&point(2)).unwrap();
        drop(store);

        let (_, points) = HistoryStore::open(&path, 10).unwrap();
        assert_eq!(points.iter().map(|p| p.t).collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn appending_never_rewrites_the_file_by_itself() {
        let path = temp_path("no-inline-compact");
        let (mut store, _) = HistoryStore::open(&path, 3).unwrap();
        for t in 1..=10 {
            store.append(&point(t)).unwrap();
        }
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 10);
        assert!(store.take_compaction().is_some());
        assert!(store.take_compaction().is_none());
    }

    #[test]
    fn points_appended_during_a_compaction_are_kept() {
        let path = temp_path("during-compact");
        let (mut store, _) = HistoryStore::open(&path, 3).unwrap();
        for t in 1..=7 {
            store.append(&point(t)).unwrap();
        }
        let job = store.take_compaction().unwrap();
        store.append(&point(8)).unwrap();
        let outcome = job.run();
        store.append(&point(9)).unwrap();
        store.finish_compaction(outcome).unwrap();
        store.append(&point(10)).unwrap();
        drop(store);

        let on_disk: Vec<u64> = HistoryStore::read_all(&path).unwrap().iter().map(|p| p.t).collect();
        assert_eq!(on_disk, vec![5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn a_failed_compaction_still_flushes_waiting_points() {
        let path = temp_path("failed-compact");
        let (mut store, _) = HistoryStore::open(&path, 1).unwrap();
        for t in 1..=3 {
            store.append(&point(t)).unwrap();
        }
        let _job = store.take_compaction().unwrap();
        store.append(&point(4)).unwrap();
        store.finish_compaction(Err(std::io::Error::other("busy"))).unwrap();
        drop(store);

        let on_disk: Vec<u64> = HistoryStore::read_all(&path).unwrap().iter().map(|p| p.t).collect();
        assert_eq!(on_disk, vec![1, 2, 3, 4]);
    }
}
