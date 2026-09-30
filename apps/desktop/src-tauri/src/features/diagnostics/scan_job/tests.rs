use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use crate::features::jobs::{GameFlag, JobState, Registry, Sleeper};
use dp_diagnostics::vpk;

const LEAKY_JS: &str = "var h = null;\nfunction f() { h = null; h = $.Schedule(1, f); }\nf();\n";
const WAIT: Duration = Duration::from_secs(5);

/// A resource with one `DATA` block: size, header version, version, table offset, table count,
/// then the table entry (tag, offset relative to the offset field, size) and the payload.
fn vjs_c(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(0u32.to_le_bytes());
    out.extend(12u16.to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(8u32.to_le_bytes());
    out.extend(1u32.to_le_bytes());
    out.extend(b"DATA");
    out.extend(8u32.to_le_bytes());
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);
    let len = out.len() as u32;
    out[..4].copy_from_slice(&len.to_le_bytes());
    out
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dlp-scanjob-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_addon(dir: &Path, file: &str) {
    let script = vjs_c(LEAKY_JS.as_bytes());
    std::fs::write(dir.join(file), vpk::write(&[("panorama/scripts/a.vjs_c", &script)])).unwrap();
}

fn registry(game: &Arc<GameFlag>) -> Arc<Registry> {
    let sleeper: Sleeper = Arc::new(|_| {});
    Registry::new(game.clone(), sleeper)
}

fn wait_until(mut ready: impl FnMut() -> bool) {
    let end = Instant::now() + WAIT;
    while !ready() {
        assert!(Instant::now() < end, "timed out waiting for condition");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn scans_every_addon_and_finishes_the_job() {
    let dir = scratch("all");
    write_addon(&dir, "pak01_dir.vpk");
    write_addon(&dir, "pak02_dir.vpk");
    let reg = registry(&GameFlag::new(false));
    let state = AddonScanState::default();
    let handle = reg.register(JOB);
    run(Some(&dir), &handle, &state);

    let report = state.report();
    assert_eq!(report.listing.as_ref().map(|l| l.addons.len()), Some(2));
    assert_eq!(report.scans.len(), 2);
    assert!(report.failures.is_empty());
    let info = reg.get(JOB.id).unwrap();
    assert_eq!(info.state, JobState::Done);
    assert_eq!((info.done, info.total), (2, 2));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_bad_addon_is_recorded_and_the_rest_still_scan() {
    let dir = scratch("bad");
    write_addon(&dir, "pak01_dir.vpk");
    std::fs::write(dir.join("pak02_dir.vpk"), b"not a vpk at all").unwrap();
    write_addon(&dir, "pak03_dir.vpk");
    let reg = registry(&GameFlag::new(false));
    let state = AddonScanState::default();
    run(Some(&dir), &reg.register(JOB), &state);

    let report = state.report();
    assert_eq!(report.scans.len(), 2);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].file_name, "pak02_dir.vpk");
    assert_eq!(reg.get(JOB.id).unwrap().state, JobState::Done);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn no_addons_folder_finishes_with_an_empty_report() {
    let reg = registry(&GameFlag::new(false));
    let state = AddonScanState::default();
    run(None, &reg.register(JOB), &state);

    let report = state.report();
    assert!(report.scans.is_empty() && report.failures.is_empty());
    assert_eq!(reg.get(JOB.id).unwrap().state, JobState::Done);
}

#[test]
fn a_cancelled_job_scans_nothing() {
    let dir = scratch("cancel");
    write_addon(&dir, "pak01_dir.vpk");
    let reg = registry(&GameFlag::new(false));
    let state = AddonScanState::default();
    let handle = reg.register(JOB);
    reg.cancel(JOB.id);
    run(Some(&dir), &handle, &state);

    assert!(state.report().scans.is_empty());
    assert_eq!(reg.get(JOB.id).unwrap().state, JobState::Cancelled);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pauses_while_the_game_runs_and_finishes_after_it_exits() {
    let dir = scratch("pause");
    write_addon(&dir, "pak01_dir.vpk");
    write_addon(&dir, "pak02_dir.vpk");
    let game = GameFlag::new(true);
    let reg = registry(&game);
    let state = Arc::new(AddonScanState::default());
    let handle = reg.register(JOB);
    let worker = {
        let (dir, state) = (dir.clone(), state.clone());
        thread::spawn(move || run(Some(&dir), &handle, &state))
    };

    wait_until(|| reg.get(JOB.id).unwrap().state == JobState::Paused);
    assert!(state.report().scans.is_empty());

    game.set(false);
    worker.join().unwrap();
    assert_eq!(state.report().scans.len(), 2);
    assert_eq!(reg.get(JOB.id).unwrap().state, JobState::Done);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_forced_scan_runs_to_the_end_while_the_game_is_running() {
    let dir = scratch("forced");
    write_addon(&dir, "pak01_dir.vpk");
    write_addon(&dir, "pak02_dir.vpk");
    let game = GameFlag::new(true);
    let reg = registry(&game);
    let state = AddonScanState::default();
    let handle = reg.register(JOB);
    reg.force_run(JOB.id);
    run(Some(&dir), &handle, &state);

    assert_eq!(state.report().scans.len(), 2);
    assert_eq!(reg.get(JOB.id).unwrap().state, JobState::Done);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn reset_clears_the_previous_report() {
    let dir = scratch("reset");
    write_addon(&dir, "pak01_dir.vpk");
    let reg = registry(&GameFlag::new(false));
    let state = AddonScanState::default();
    run(Some(&dir), &reg.register(JOB), &state);
    assert_eq!(state.report().scans.len(), 1);

    state.reset();
    let report = state.report();
    assert!(report.listing.is_none() && report.scans.is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}
