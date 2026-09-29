use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use ts_rs::TS;

use serde::Serialize;

use dp_sync::LockExt;

const DEADLOCK_APP_ID: &str = "1422450";

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VoiceBanFile {
    pub path: String,
    pub exists: bool,
    pub text: String,
    pub game_running: bool,
}

pub fn voice_ban_path(userdata_dir: &Path) -> PathBuf {
    userdata_dir.join(DEADLOCK_APP_ID).join("remote").join("voice_ban.dt")
}

fn looks_like_voice_ban(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok_and(|t| t.contains("users"))
}

fn backup_candidate(path: &Path, timestamp: u64, n: u32) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".backup-{timestamp}"));
    if n > 0 {
        name.push(format!("-{n}"));
    }
    path.with_file_name(name)
}

/// Copies `path` to a fresh `<path>.backup-<timestamp>[-n]`, then overwrites it. The original is
/// never touched unless the backup succeeded. The suffix keeps two writes in one second from
/// replacing the first backup with an already-edited copy.
pub fn backup_then_write(path: &Path, bytes: &[u8], timestamp: u64) -> Result<PathBuf, String> {
    if !path.is_file() {
        log::error!("mute list write refused: voice_ban.dt does not exist");
        return Err(format!("No file exists at {}. Not creating one.", path.display()));
    }
    let backup = (0..64)
        .map(|n| backup_candidate(path, timestamp, n))
        .find(|p| !p.exists())
        .ok_or("Could not find a free backup name. The file was not changed.")?;
    std::fs::copy(path, &backup).map_err(|e| {
        log::error!("mute list backup failed, the file was not changed: {e}");
        format!("Backup failed, the file was not changed: {e}")
    })?;
    dp_atomic::write_atomic(path, bytes).map_err(|e| {
        log::error!("mute list write failed after the backup was saved: {e}");
        format!("Backup saved to {}, but writing the file failed: {e}. The original is unchanged.", backup.display())
    })?;
    log::info!("mute list written ({} bytes), backup saved", bytes.len());
    Ok(backup)
}

pub(crate) fn game_running() -> bool {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys.processes().values().any(|p| crate::features::game::is_process(p.name()))
}

static LAST_CHECK: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

fn cached_check(
    slot: &Mutex<Option<(Instant, bool)>>,
    now: Instant,
    max_age: Duration,
    probe: impl FnOnce() -> bool,
) -> bool {
    let mut last = slot.lock_or_recover();
    match *last {
        Some((at, value)) if now.saturating_duration_since(at) < max_age => value,
        _ => {
            let value = probe();
            *last = Some((now, value));
            value
        }
    }
}

/// The page polls for the lock state, and a full process scan per poll is wasteful. A write still
/// uses the uncached check, so a stale answer can never let one through.
pub(crate) fn game_running_recent() -> bool {
    cached_check(&LAST_CHECK, Instant::now(), Duration::from_secs(5), game_running)
}

fn current_path() -> Result<PathBuf, String> {
    let account = crate::features::steam_account::current_account().ok_or("No Steam account found.")?;
    let dir = account.userdata_dir.ok_or("This account has no Steam userdata folder.")?;
    Ok(voice_ban_path(Path::new(&dir)))
}

pub mod commands {
    use super::{
        backup_then_write, current_path, game_running, game_running_recent, looks_like_voice_ban, VoiceBanFile,
    };

    #[tauri::command]
    pub fn is_game_running() -> bool {
        game_running_recent()
    }

    #[tauri::command]
    pub fn read_voice_ban() -> Result<VoiceBanFile, String> {
        let path = current_path()?;
        let game_running = game_running_recent();
        let (exists, text) = match std::fs::read(&path) {
            Ok(bytes) => (true, String::from_utf8(bytes).map_err(|_| "voice_ban.dt is not valid UTF-8.".to_string())?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (false, String::new()),
            Err(e) => {
                log::error!("could not read voice_ban.dt: {e}");
                return Err(format!("Failed to read {}: {e}", path.display()));
            }
        };
        Ok(VoiceBanFile { path: path.to_string_lossy().into_owned(), exists, text, game_running })
    }

    /// The target is always the current account's own file; the frontend never supplies a path.
    #[tauri::command]
    pub fn write_voice_ban(text: String) -> Result<String, String> {
        if game_running() {
            log::warn!("mute list write refused: Deadlock is running");
            return Err("Deadlock is running. Close the game before changing mutes.".into());
        }
        if !looks_like_voice_ban(text.as_bytes()) {
            log::error!("mute list write refused: the content is not a mute list");
            return Err("Refusing to write: the content is not a mute list.".into());
        }
        let path = current_path()?;
        let timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
        backup_then_write(&path, text.as_bytes(), timestamp).map(|p| p.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-vb-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_recent_answer_is_reused_and_an_old_one_is_probed_again() {
        let slot = Mutex::new(None);
        let t0 = Instant::now();
        let age = Duration::from_secs(5);
        assert!(cached_check(&slot, t0, age, || true));
        assert!(cached_check(&slot, t0 + Duration::from_secs(4), age, || panic!("probed within the window")));
        assert!(!cached_check(&slot, t0 + Duration::from_secs(6), age, || false));
    }

    #[test]
    fn path_is_under_the_deadlock_remote_folder() {
        let p = voice_ban_path(Path::new("U"));
        assert_eq!(p, Path::new("U").join("1422450").join("remote").join("voice_ban.dt"));
    }

    #[test]
    fn writes_new_bytes_and_keeps_a_backup_of_the_original() {
        let dir = temp_dir("ok");
        let file = dir.join("voice_ban.dt");
        fs::write(&file, "old").unwrap();
        let backup = backup_then_write(&file, b"new", 42).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "new");
        assert_eq!(fs::read_to_string(&backup).unwrap(), "old");
        assert!(backup.file_name().unwrap().to_string_lossy().ends_with(".backup-42"));
    }

    #[test]
    fn a_second_write_in_the_same_second_keeps_the_first_backup() {
        let dir = temp_dir("twice");
        let file = dir.join("voice_ban.dt");
        fs::write(&file, "v1").unwrap();
        let first = backup_then_write(&file, b"v2", 7).unwrap();
        let second = backup_then_write(&file, b"v3", 7).unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(&first).unwrap(), "v1");
        assert_eq!(fs::read_to_string(&second).unwrap(), "v2");
    }

    #[test]
    fn refuses_to_create_a_file_that_does_not_exist() {
        let dir = temp_dir("missing");
        let file = dir.join("voice_ban.dt");
        assert!(backup_then_write(&file, b"x", 1).is_err());
        assert!(!file.exists());
    }

    #[test]
    fn leaves_the_original_alone_when_the_backup_fails() {
        let dir = temp_dir("blocked");
        let file = dir.join("voice_ban.dt");
        fs::write(&file, "old").unwrap();
        // A directory squatting on every candidate backup name makes the copy fail.
        for n in 0..64 {
            let name = if n == 0 { "voice_ban.dt.backup-9".to_string() } else { format!("voice_ban.dt.backup-9-{n}") };
            fs::create_dir(dir.join(name)).unwrap();
        }
        assert!(backup_then_write(&file, b"new", 9).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), "old");
    }

    #[test]
    fn only_kv3_text_with_a_users_list_is_writable() {
        assert!(looks_like_voice_ban(b"{\n\tusers = null\n}"));
        assert!(looks_like_voice_ban(b"{ users = [ ] }"));
        assert!(!looks_like_voice_ban(b""));
        assert!(!looks_like_voice_ban(b"garbage"));
    }
}
