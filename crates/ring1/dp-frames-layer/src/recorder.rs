use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use dp_frames_wire::{encode_header, encode_record, Record, Ring, FRAME_FILE_EXTENSION};

const RING_CAPACITY: usize = 1 << 14;
const IDLE_SLEEP: Duration = Duration::from_millis(3);
const FRAMES_DIR_ENV: &str = "DEADLOCK_PLUS_FRAMES_DIR";

/// Hands present timestamps from the render thread to a writer thread through a lock-free ring, so the
/// render thread never waits on the disk.
pub struct Recorder {
    ring: Arc<Ring>,
    failed: Arc<AtomicBool>,
}

impl Recorder {
    pub fn start(dir: &Path, pid: u32) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let mut file = File::create(dir.join(format!("{pid}.{FRAME_FILE_EXTENSION}")))?;
        file.write_all(&encode_header(pid))?;
        let ring = Arc::new(Ring::new(RING_CAPACITY));
        let failed = Arc::new(AtomicBool::new(false));
        let (thread_ring, thread_failed) = (ring.clone(), failed.clone());
        thread::Builder::new().name("deadlock-plus-frames".into()).spawn(move || {
            let mut bytes = Vec::with_capacity(RING_CAPACITY * 16);
            loop {
                bytes.clear();
                while let Some((timestamp_ns, swapchain)) = thread_ring.pop() {
                    encode_record(Record { timestamp_ns, swapchain }, &mut bytes);
                }
                if bytes.is_empty() {
                    thread::sleep(IDLE_SLEEP);
                } else if file.write_all(&bytes).is_err() {
                    thread_failed.store(true, Ordering::Relaxed);
                    return;
                }
            }
        })?;
        Ok(Self { ring, failed })
    }

    pub fn record(&self, timestamp_ns: u64, swapchain: u32) {
        if !self.failed.load(Ordering::Relaxed) {
            self.ring.push(timestamp_ns, swapchain);
        }
    }
}

static RECORDER: OnceLock<Option<Recorder>> = OnceLock::new();

/// Starts the process-wide recorder once. Failure leaves the layer as a pass-through.
pub fn init() {
    RECORDER.get_or_init(|| {
        let dir = frames_dir_from_env()?;
        Recorder::start(&dir, std::process::id()).ok()
    });
}

pub fn record(timestamp_ns: u64, swapchain: u32) {
    if let Some(Some(recorder)) = RECORDER.get() {
        recorder.record(timestamp_ns, swapchain);
    }
}

fn frames_dir_from_env() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(FRAMES_DIR_ENV).filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    dp_frames_wire::data_home(xdg.as_deref(), home.as_deref()).map(|d| dp_frames_wire::frames_dir(&d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{read_until, temp_dir};

    #[test]
    fn records_reach_the_file_in_order() {
        let dir = temp_dir("recorder");
        let recorder = Recorder::start(&dir, 4242).unwrap();
        for i in 1..=100u64 {
            recorder.record(i * 1000, 9);
        }
        let out = read_until(&dir.join("4242.dpf"), 100);
        assert_eq!(out.len(), 100);
        assert!(out.iter().enumerate().all(|(i, r)| r.timestamp_ns == (i as u64 + 1) * 1000 && r.swapchain == 9));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unwritable_folder_is_an_error_not_a_panic() {
        let dir = temp_dir("blocked");
        std::fs::create_dir_all(&dir).unwrap();
        let file_in_the_way = dir.join("file");
        std::fs::write(&file_in_the_way, b"x").unwrap();
        assert!(Recorder::start(&file_in_the_way.join("sub"), 1).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
