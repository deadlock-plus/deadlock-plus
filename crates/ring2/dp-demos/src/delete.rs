use std::path::{Path, PathBuf};
use ts_rs::TS;

use serde::Serialize;

use super::is_deletable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RecycleAvailability {
    Available,
    TooLarge,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureReason {
    NotAReplay,
    Missing,
    Pinned,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub file_name: String,
    pub reason: FailureReason,
}

#[derive(Debug, Default)]
pub struct DeleteReport {
    pub deleted: Vec<String>,
    pub failed: Vec<Failure>,
}

/// Windows deletes items that would overflow the bin outright instead of recycling them, so
/// the check has to happen before anything is sent there.
pub fn recycle_availability(incoming: u64, bin_used: u64, limit: u64, disabled: bool) -> RecycleAvailability {
    if disabled {
        RecycleAvailability::Disabled
    } else if bin_used.saturating_add(incoming) > limit {
        RecycleAvailability::TooLarge
    } else {
        RecycleAvailability::Available
    }
}

/// Windows' stock bin size when a drive has no custom limit is a small share of the drive.
pub fn default_limit(volume_total: u64) -> u64 {
    volume_total / 20
}

/// Only file names are accepted from the frontend; each is re-joined onto the replays folder and
/// must pass the same guard as everything else.
pub fn resolve_targets(dir: &Path, names: &[String]) -> (Vec<Target>, Vec<Failure>) {
    let (mut targets, mut failed) = (Vec::new(), Vec::new());
    for name in names {
        let path = dir.join(name);
        let fail = |reason| Failure { file_name: name.clone(), reason };
        if !is_deletable(dir, &path) {
            failed.push(fail(FailureReason::NotAReplay));
            continue;
        }
        match std::fs::metadata(&path) {
            Ok(m) if m.is_file() => targets.push(Target { file_name: name.clone(), path, size: m.len() }),
            _ => failed.push(fail(FailureReason::Missing)),
        }
    }
    (targets, failed)
}

pub fn delete_permanently(targets: Vec<Target>) -> DeleteReport {
    let mut report = DeleteReport::default();
    for t in targets {
        match std::fs::remove_file(&t.path) {
            Ok(()) => report.deleted.push(t.file_name),
            Err(e) => {
                log::warn!("could not delete {}: {e}", t.file_name);
                report.failed.push(Failure { file_name: t.file_name, reason: FailureReason::Io });
            }
        }
    }
    report
}

#[derive(Debug, Clone, Copy)]
pub struct BinInfo {
    pub used: u64,
    pub limit: u64,
    pub disabled: bool,
}

#[cfg(windows)]
pub fn bin_info(dir: &Path) -> Option<BinInfo> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetVolumeNameForVolumeMountPointW, GetVolumePathNameW,
    };
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    use windows::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};

    fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        s.encode_wide().chain(std::iter::once(0)).collect()
    }
    fn text(buf: &[u16]) -> String {
        String::from_utf16_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())])
    }

    let dir_w = wide(dir.as_os_str());
    let mut root = [0u16; 260];
    unsafe { GetVolumePathNameW(PCWSTR(dir_w.as_ptr()), &mut root).ok()? };

    let mut volume = [0u16; 64];
    unsafe { GetVolumeNameForVolumeMountPointW(PCWSTR(root.as_ptr()), &mut volume).ok()? };
    let guid = text(&volume);
    let guid = guid.trim_start_matches(r"\\?\Volume").trim_end_matches('\\').to_string();

    let mut total = 0u64;
    unsafe { GetDiskFreeSpaceExW(PCWSTR(root.as_ptr()), None, Some(&mut total), None).ok()? };

    let mut info = SHQUERYRBINFO { cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };
    unsafe { SHQueryRecycleBinW(PCWSTR(root.as_ptr()), &mut info).ok()? };

    let subkey = wide(std::ffi::OsStr::new(&format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\BitBucket\Volume\{guid}"
    )));
    let dword = |name: &str| -> Option<u32> {
        let name_w = wide(std::ffi::OsStr::new(name));
        let mut value = 0u32;
        let mut size = 4u32;
        let err = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(name_w.as_ptr()),
                RRF_RT_REG_DWORD,
                None,
                Some(&mut value as *mut u32 as *mut _),
                Some(&mut size),
            )
        };
        err.is_ok().then_some(value)
    };
    let limit = dword("MaxCapacity").map_or_else(|| default_limit(total), |mb| u64::from(mb) * 1024 * 1024);
    Some(BinInfo { used: info.i64Size.max(0) as u64, limit, disabled: dword("NukeOnDelete") == Some(1) })
}

#[cfg(not(windows))]
pub fn bin_info(_dir: &Path) -> Option<BinInfo> {
    None
}

#[cfg(windows)]
pub fn availability_for(dir: &Path, incoming: u64) -> RecycleAvailability {
    match bin_info(dir) {
        Some(b) => recycle_availability(incoming, b.used, b.limit, b.disabled),
        None => RecycleAvailability::TooLarge,
    }
}

/// The freedesktop and macOS trash have no size cap to check against.
#[cfg(not(windows))]
pub fn availability_for(_dir: &Path, _incoming: u64) -> RecycleAvailability {
    RecycleAvailability::Available
}

pub fn delete_to_bin(targets: Vec<Target>) -> DeleteReport {
    let mut report = DeleteReport::default();
    for t in targets {
        match trash::delete(&t.path) {
            Ok(()) => report.deleted.push(t.file_name),
            Err(e) => {
                log::warn!("could not recycle {}: {e}", t.file_name);
                report.failed.push(Failure { file_name: t.file_name, reason: FailureReason::Io });
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-del-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn recycle_needs_room_for_the_new_files_on_top_of_what_the_bin_holds() {
        assert_eq!(recycle_availability(10, 0, 100, false), RecycleAvailability::Available);
        assert_eq!(recycle_availability(10, 90, 100, false), RecycleAvailability::Available);
        assert_eq!(recycle_availability(11, 90, 100, false), RecycleAvailability::TooLarge);
        assert_eq!(recycle_availability(101, 0, 100, false), RecycleAvailability::TooLarge);
    }

    #[test]
    fn a_disabled_bin_wins_over_size() {
        assert_eq!(recycle_availability(1, 0, 100, true), RecycleAvailability::Disabled);
    }

    #[cfg(not(windows))]
    #[test]
    fn trash_off_windows_is_always_available() {
        assert_eq!(availability_for(Path::new("/"), u64::MAX), RecycleAvailability::Available);
    }

    #[test]
    fn the_default_limit_is_a_twentieth_of_the_drive() {
        assert_eq!(default_limit(2000), 100);
    }

    #[test]
    fn only_existing_replay_names_become_targets() {
        let dir = temp_dir("resolve");
        fs::write(dir.join("1.dem"), b"abc").unwrap();
        fs::write(dir.join("2.dem.partial"), b"z").unwrap();
        fs::write(dir.join("keep.txt"), b"x").unwrap();
        let (targets, failed) = resolve_targets(
            &dir,
            &names(&["1.dem", "2.dem.partial", "3.dem", "keep.txt", "..\\1.dem", "../1.dem", "sub/1.dem"]),
        );
        let ok: Vec<_> = targets.iter().map(|t| (t.file_name.as_str(), t.size)).collect();
        assert_eq!(ok, vec![("1.dem", 3), ("2.dem.partial", 1)]);
        assert_eq!(failed.len(), 5);
        let reasons: Vec<_> = failed.iter().map(|f| (f.file_name.as_str(), f.reason)).collect();
        assert!(reasons.contains(&("3.dem", FailureReason::Missing)));
        assert!(reasons.contains(&("keep.txt", FailureReason::NotAReplay)));
    }

    #[test]
    fn permanent_delete_removes_files_and_reports_the_ones_it_could_not() {
        let dir = temp_dir("perm");
        fs::write(dir.join("1.dem"), b"abc").unwrap();
        let (mut targets, _) = resolve_targets(&dir, &names(&["1.dem"]));
        targets.push(Target { file_name: "9.dem".into(), path: dir.join("9.dem"), size: 0 });
        let report = delete_permanently(targets);
        assert_eq!(report.deleted, vec!["1.dem".to_string()]);
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].file_name, "9.dem");
        assert_eq!(report.failed[0].reason, FailureReason::Io);
        assert!(!dir.join("1.dem").exists());
    }

    #[test]
    fn other_files_in_the_folder_are_never_touched() {
        let dir = temp_dir("safe");
        fs::write(dir.join("1.dem"), b"a").unwrap();
        fs::write(dir.join("notes.txt"), b"n").unwrap();
        let (targets, _) = resolve_targets(&dir, &names(&["1.dem", "notes.txt"]));
        delete_permanently(targets);
        assert!(dir.join("notes.txt").exists());
    }
}
