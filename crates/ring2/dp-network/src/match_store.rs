use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::range_stats::points_in_range;
use crate::types::HistoryPoint;

const MAGIC: [u8; 4] = *b"DPMP";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 16;
const ICMP_RECORD_LEN: usize = 6;
const ENGINE_RECORD_LEN: usize = 10;
const FILE_EXT: &str = "dpmp";
const INDEX_FILE: &str = "index.json";
const FLAG_PARTIAL: u8 = 1;
const NO_PING: u16 = u16::MAX;
const MAX_QUANTISED: u16 = u16::MAX - 1;

/// Where the samples of a recording came from. It fixes the record layout of the whole file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingSource {
    /// Read from the game's own network channel: ping plus loss and jitter.
    Engine,
    /// ICMP round trip to the relay: ping only, `None` is a timeout.
    Icmp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineStats {
    /// Fraction of packets lost inbound, 0.0..=1.0.
    pub loss_down: f32,
    /// Fraction of packets lost outbound, 0.0..=1.0.
    pub loss_up: f32,
    pub jitter_ms: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchSample {
    /// Milliseconds since the recording's `start_ms`.
    pub offset_ms: u32,
    /// `None` when no ping was measured (ICMP timeout).
    pub ping_ms: Option<f32>,
    /// Present exactly when the recording's source is `Engine`.
    pub engine: Option<EngineStats>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchHeader {
    pub version: u8,
    pub source: PingSource,
    /// The recording started after the match had already begun.
    pub partial: bool,
    /// Unix milliseconds of the recording's time zero.
    pub start_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchPing {
    pub header: MatchHeader,
    pub points: Vec<MatchSample>,
}

impl MatchPing {
    /// Absolute-time ping curve in the shape the history queries use.
    pub fn history_points(&self) -> Vec<HistoryPoint> {
        self.points
            .iter()
            .map(|s| HistoryPoint { t: self.header.start_ms + u64::from(s.offset_ms), raw: s.ping_ms })
            .collect()
    }

    /// At most `max_points` points, each bucket keeping its worst ping (the rule `points_in_range` applies).
    /// Loss and jitter are not carried.
    pub fn downsample(&self, max_points: usize) -> Vec<HistoryPoint> {
        points_in_range(&self.history_points(), 0, u64::MAX, max_points)
    }
}

fn record_len(source: PingSource) -> usize {
    match source {
        PingSource::Icmp => ICMP_RECORD_LEN,
        PingSource::Engine => ENGINE_RECORD_LEN,
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_owned())
}

fn tenths(ms: f32) -> u16 {
    (ms.max(0.0) * 10.0).round().min(f32::from(MAX_QUANTISED)) as u16
}

fn fraction(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn encode_header(source: PingSource, partial: bool, start_ms: u64) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[..4].copy_from_slice(&MAGIC);
    out[4] = VERSION;
    out[5] = match source {
        PingSource::Engine => 0,
        PingSource::Icmp => 1,
    };
    out[6] = if partial { FLAG_PARTIAL } else { 0 };
    out[8..].copy_from_slice(&start_ms.to_le_bytes());
    out
}

/// `None` for a short header, a foreign file, or a version this build does not know.
fn decode_header(bytes: &[u8]) -> Option<MatchHeader> {
    if bytes.len() < HEADER_LEN || bytes[..4] != MAGIC || bytes[4] != VERSION {
        return None;
    }
    let source = match bytes[5] {
        0 => PingSource::Engine,
        1 => PingSource::Icmp,
        _ => return None,
    };
    let start_ms = u64::from_le_bytes(bytes[8..HEADER_LEN].try_into().ok()?);
    Some(MatchHeader { version: bytes[4], source, partial: bytes[6] & FLAG_PARTIAL != 0, start_ms })
}

fn decode_sample(source: PingSource, record: &[u8]) -> MatchSample {
    let offset_ms = u32::from_le_bytes([record[0], record[1], record[2], record[3]]);
    let ping = u16::from_le_bytes([record[4], record[5]]);
    let engine = (source == PingSource::Engine).then(|| EngineStats {
        loss_down: f32::from(record[6]) / 255.0,
        loss_up: f32::from(record[7]) / 255.0,
        jitter_ms: f32::from(u16::from_le_bytes([record[8], record[9]])) / 10.0,
    });
    MatchSample { offset_ms, ping_ms: (ping != NO_PING).then(|| f32::from(ping) / 10.0), engine }
}

/// An open, append-only recording. Dropping it without `finish` leaves a valid file.
pub struct MatchRecording {
    session_key: String,
    source: PingSource,
    file: File,
}

impl MatchRecording {
    pub fn session_key(&self) -> &str {
        &self.session_key
    }

    /// Writes one record in a single call, so a crash can only tear the last record.
    pub fn append(&mut self, sample: &MatchSample) -> io::Result<()> {
        if sample.engine.is_some() != (self.source == PingSource::Engine) {
            return Err(invalid("sample does not match the recording source"));
        }
        let mut record = [0u8; ENGINE_RECORD_LEN];
        record[..4].copy_from_slice(&sample.offset_ms.to_le_bytes());
        let ping = sample.ping_ms.map_or(NO_PING, tenths);
        record[4..6].copy_from_slice(&ping.to_le_bytes());
        if let Some(e) = sample.engine {
            record[6] = fraction(e.loss_down);
            record[7] = fraction(e.loss_up);
            record[8..10].copy_from_slice(&tenths(e.jitter_ms).to_le_bytes());
        }
        self.file.write_all(&record[..record_len(self.source)])
    }

    pub fn finish(mut self) -> io::Result<()> {
        self.file.flush()?;
        self.file.sync_all()
    }
}

fn read_file(path: &Path) -> io::Result<Option<MatchPing>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let Some(header) = decode_header(&bytes) else {
        log::warn!("ignoring unreadable match ping file {}", path.display());
        return Ok(None);
    };
    let points = bytes[HEADER_LEN..]
        .chunks_exact(record_len(header.source))
        .map(|record| decode_sample(header.source, record))
        .collect();
    Ok(Some(MatchPing { header, points }))
}

/// A folder of per-match ping files plus an index from match id to file.
///
/// Calls are not synchronised: the owner serialises `bind_match_id`, `prune_older_than` and recording starts.
pub struct MatchStore {
    dir: PathBuf,
}

impl MatchStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn session_path(&self, session_key: &str) -> io::Result<PathBuf> {
        let valid =
            !session_key.is_empty() && session_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !valid {
            return Err(invalid("session key may only hold letters, digits, '-' and '_'"));
        }
        Ok(self.dir.join(format!("{session_key}.{FILE_EXT}")))
    }

    fn load_index(&self) -> HashMap<String, String> {
        fs::read(self.dir.join(INDEX_FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    fn save_index(&self, index: &HashMap<String, String>) -> io::Result<()> {
        dp_atomic::write_atomic(&self.dir.join(INDEX_FILE), &serde_json::to_vec(index)?)
    }

    /// Creates the file for a new recording under a temporary `session_key` (letters, digits, `-`, `_`).
    pub fn start(
        &self,
        session_key: &str,
        source: PingSource,
        partial: bool,
        start_ms: u64,
    ) -> io::Result<MatchRecording> {
        let path = self.session_path(session_key)?;
        fs::create_dir_all(&self.dir)?;
        let mut file = OpenOptions::new().append(true).create_new(true).open(path)?;
        file.write_all(&encode_header(source, partial, start_ms))?;
        Ok(MatchRecording { session_key: session_key.to_owned(), source, file })
    }

    /// Points the match id at the session's file. `Ok(false)` when no such session file exists.
    pub fn bind_match_id(&self, session_key: &str, match_id: u64) -> io::Result<bool> {
        if !self.session_path(session_key)?.is_file() {
            return Ok(false);
        }
        let mut index = self.load_index();
        index.insert(match_id.to_string(), session_key.to_owned());
        self.save_index(&index)?;
        Ok(true)
    }

    /// `None` for an unbound id, a missing file, or a file written by an unknown format version.
    pub fn read(&self, match_id: u64) -> io::Result<Option<MatchPing>> {
        match self.load_index().get(&match_id.to_string()) {
            Some(session_key) => self.read_session(session_key),
            None => Ok(None),
        }
    }

    pub fn read_session(&self, session_key: &str) -> io::Result<Option<MatchPing>> {
        read_file(&self.session_path(session_key)?)
    }

    /// Deletes recordings whose file was last written more than `age` ago and drops their index entries.
    /// Returns how many files went. Never called automatically.
    pub fn prune_older_than(&self, age: Duration) -> io::Result<usize> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };
        let cutoff = SystemTime::now().checked_sub(age).unwrap_or(SystemTime::UNIX_EPOCH);
        let mut removed = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some(FILE_EXT) {
                continue;
            }
            if fs::metadata(&path)?.modified()? < cutoff {
                fs::remove_file(&path)?;
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    removed.push(stem.to_owned());
                }
            }
        }
        if !removed.is_empty() {
            let mut index = self.load_index();
            index.retain(|_, session| !removed.contains(session));
            self.save_index(&index)?;
        }
        Ok(removed.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> (MatchStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-matchstore-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        (MatchStore::new(&dir), dir)
    }

    fn icmp(offset_ms: u32, ping: Option<f32>) -> MatchSample {
        MatchSample { offset_ms, ping_ms: ping, engine: None }
    }

    fn engine(offset_ms: u32, ping: f32, down: f32, up: f32, jitter: f32) -> MatchSample {
        MatchSample {
            offset_ms,
            ping_ms: Some(ping),
            engine: Some(EngineStats { loss_down: down, loss_up: up, jitter_ms: jitter }),
        }
    }

    #[test]
    fn engine_samples_round_trip_by_match_id() {
        let (store, _) = temp_store("engine");
        let mut rec = store.start("s1", PingSource::Engine, false, 1_700_000_000_000).unwrap();
        rec.append(&engine(0, 42.0, 0.0, 0.0, 2.5)).unwrap();
        rec.append(&engine(1000, 55.5, 0.25, 0.5, 6.0)).unwrap();
        rec.finish().unwrap();
        assert!(store.bind_match_id("s1", 777).unwrap());

        let got = store.read(777).unwrap().unwrap();
        assert_eq!(got.header.source, PingSource::Engine);
        assert_eq!(got.header.start_ms, 1_700_000_000_000);
        assert!(!got.header.partial);
        assert_eq!(got.points.len(), 2);
        assert_eq!(got.points[1].offset_ms, 1000);
        assert_eq!(got.points[1].ping_ms, Some(55.5));
        let e = got.points[1].engine.unwrap();
        assert!((e.loss_down - 0.25).abs() < 0.005);
        assert!((e.loss_up - 0.5).abs() < 0.005);
        assert!((e.jitter_ms - 6.0).abs() < 0.05);
    }

    #[test]
    fn icmp_timeout_is_kept_as_none() {
        let (store, _) = temp_store("icmp");
        let mut rec = store.start("s1", PingSource::Icmp, false, 5).unwrap();
        rec.append(&icmp(0, Some(30.0))).unwrap();
        rec.append(&icmp(1000, None)).unwrap();
        rec.finish().unwrap();

        let got = store.read_session("s1").unwrap().unwrap();
        assert_eq!(got.points[0].ping_ms, Some(30.0));
        assert_eq!(got.points[1].ping_ms, None);
        assert!(got.points[1].engine.is_none());
    }

    #[test]
    fn a_torn_trailing_record_is_ignored() {
        let (store, dir) = temp_store("torn");
        let mut rec = store.start("s1", PingSource::Icmp, false, 0).unwrap();
        rec.append(&icmp(0, Some(10.0))).unwrap();
        rec.append(&icmp(1000, Some(20.0))).unwrap();
        rec.finish().unwrap();
        let path = dir.join("s1.dpmp");
        let len = fs::metadata(&path).unwrap().len();
        OpenOptions::new().write(true).open(&path).unwrap().set_len(len - 3).unwrap();

        let got = store.read_session("s1").unwrap().unwrap();
        assert_eq!(got.points.len(), 1);
        assert_eq!(got.points[0].ping_ms, Some(10.0));
    }

    #[test]
    fn a_torn_header_reads_as_none() {
        let (store, dir) = temp_store("tornheader");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("s1.dpmp"), b"DPMP\x01").unwrap();
        assert!(store.read_session("s1").unwrap().is_none());
    }

    #[test]
    fn the_partial_flag_is_stored() {
        let (store, _) = temp_store("partial");
        store.start("s1", PingSource::Icmp, true, 9).unwrap().finish().unwrap();
        let got = store.read_session("s1").unwrap().unwrap();
        assert!(got.header.partial);
        assert!(got.points.is_empty());
    }

    #[test]
    fn a_bound_id_survives_a_new_store_instance() {
        let (store, dir) = temp_store("bind");
        store.start("tmp", PingSource::Icmp, false, 1).unwrap().finish().unwrap();
        assert!(store.read(5).unwrap().is_none());
        assert!(store.bind_match_id("tmp", 5).unwrap());
        assert!(MatchStore::new(&dir).read(5).unwrap().is_some());
    }

    #[test]
    fn unknown_ids_and_sessions_read_as_none() {
        let (store, _) = temp_store("unknown");
        assert!(store.read(1).unwrap().is_none());
        assert!(store.read_session("nope").unwrap().is_none());
        assert!(!store.bind_match_id("nope", 1).unwrap());
    }

    #[test]
    fn an_unknown_format_version_reads_as_none() {
        let (store, dir) = temp_store("version");
        store.start("s1", PingSource::Icmp, false, 1).unwrap().finish().unwrap();
        let path = dir.join("s1.dpmp");
        let mut bytes = fs::read(&path).unwrap();
        bytes[4] = 99;
        fs::write(&path, bytes).unwrap();
        assert!(store.read_session("s1").unwrap().is_none());
    }

    #[test]
    fn starting_over_an_existing_session_fails_and_keeps_the_data() {
        let (store, _) = temp_store("exists");
        let mut rec = store.start("s1", PingSource::Icmp, false, 1).unwrap();
        rec.append(&icmp(0, Some(1.0))).unwrap();
        rec.finish().unwrap();
        let err = store.start("s1", PingSource::Icmp, false, 2).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(store.read_session("s1").unwrap().unwrap().points.len(), 1);
    }

    #[test]
    fn session_keys_cannot_escape_the_folder_and_samples_must_match_the_source() {
        let (store, _) = temp_store("invalid");
        assert_eq!(store.start("../x", PingSource::Icmp, false, 1).err().unwrap().kind(), io::ErrorKind::InvalidInput);
        let mut rec = store.start("s1", PingSource::Icmp, false, 1).unwrap();
        assert_eq!(rec.append(&engine(0, 1.0, 0.0, 0.0, 0.0)).unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let mut rec = store.start("s2", PingSource::Engine, false, 1).unwrap();
        assert_eq!(rec.append(&icmp(0, Some(1.0))).unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn history_points_use_absolute_time() {
        let ping = MatchPing {
            header: MatchHeader { version: 1, source: PingSource::Icmp, partial: false, start_ms: 1000 },
            points: vec![icmp(0, Some(10.0)), icmp(500, None)],
        };
        let pts = ping.history_points();
        assert_eq!(pts.iter().map(|p| p.t).collect::<Vec<_>>(), vec![1000, 1500]);
        assert_eq!(pts[1].raw, None);
    }

    #[test]
    fn downsample_keeps_the_worst_ping_per_bucket() {
        let points = (0..1000u32).map(|i| icmp(i * 1000, Some(if i == 421 { 400.0 } else { 20.0 }))).collect();
        let ping = MatchPing {
            header: MatchHeader { version: 1, source: PingSource::Icmp, partial: false, start_ms: 0 },
            points,
        };
        let got = ping.downsample(300);
        assert!(got.len() <= 300);
        assert!(got.iter().any(|p| p.raw == Some(400.0)));
    }

    #[test]
    fn pruning_removes_old_files_and_their_index_entries() {
        let (store, dir) = temp_store("prune");
        store.start("old", PingSource::Icmp, false, 1).unwrap().finish().unwrap();
        store.bind_match_id("old", 10).unwrap();
        let old = OpenOptions::new().write(true).open(dir.join("old.dpmp")).unwrap();
        old.set_modified(SystemTime::now() - Duration::from_secs(10 * 86_400)).unwrap();
        drop(old);
        store.start("new", PingSource::Icmp, false, 2).unwrap().finish().unwrap();
        store.bind_match_id("new", 11).unwrap();

        assert_eq!(store.prune_older_than(Duration::from_secs(86_400)).unwrap(), 1);
        assert!(store.read(10).unwrap().is_none());
        assert!(store.read(11).unwrap().is_some());
        assert!(store.read_session("new").unwrap().is_some());
    }

    #[test]
    fn nothing_is_pruned_when_nothing_is_old_enough() {
        let (store, _) = temp_store("prune-none");
        store.start("a", PingSource::Icmp, false, 1).unwrap().finish().unwrap();
        assert_eq!(store.prune_older_than(Duration::from_secs(86_400)).unwrap(), 0);
        let missing = MatchStore::new(std::env::temp_dir().join("dp-missing-matchstore-xyz"));
        assert_eq!(missing.prune_older_than(Duration::ZERO).unwrap(), 0);
    }
}
