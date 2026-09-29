use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::types::HistoryPoint;

/// Append-only JSON Lines log of ping history. One point per line so a crash mid-write
/// only ever damages the last line, which the loader skips.
pub struct HistoryStore {
    path: PathBuf,
    keep: usize,
    lines: usize,
    file: File,
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
    /// Loads the newest `keep` points, rewrites the file trimmed to them (dropping corrupt
    /// lines), and opens it for appending.
    pub fn open(path: &Path, keep: usize) -> std::io::Result<(Self, Vec<HistoryPoint>)> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let points = compact(path, keep)?;
        let file = OpenOptions::new().append(true).open(path)?;
        Ok((Self { path: path.to_path_buf(), keep, lines: points.len(), file }, points))
    }

    /// The file is trimmed again once it holds twice `keep` lines, so a session that never restarts
    /// still has a bounded log. Rewriting needs the append handle closed first, hence the reopen.
    pub fn append(&mut self, point: &HistoryPoint) -> std::io::Result<()> {
        writeln!(self.file, "{}", serde_json::to_string(point)?)?;
        self.lines += 1;
        if self.lines > self.keep.saturating_mul(2) {
            self.lines = compact(&self.path, self.keep)?.len();
            self.file = OpenOptions::new().append(true).open(&self.path)?;
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
        HistoryPoint { t, raw: Some(t as f32), exit: None }
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
        assert_eq!(points[1].exit, None);
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
}
