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

pub fn looks_like_voice_ban(bytes: &[u8]) -> bool {
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

#[derive(Debug)]
pub enum WriteError {
    Missing(PathBuf),
    NoBackupName,
    BackupFailed(std::io::Error),
    WriteFailed { backup: PathBuf, source: std::io::Error },
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(path) => write!(f, "No file exists at {}. Not creating one.", path.display()),
            Self::NoBackupName => f.write_str("Could not find a free backup name. The file was not changed."),
            Self::BackupFailed(e) => write!(f, "Backup failed, the file was not changed: {e}"),
            Self::WriteFailed { backup, source } => write!(
                f,
                "Backup saved to {}, but writing the file failed: {source}. The original is unchanged.",
                backup.display()
            ),
        }
    }
}

impl std::error::Error for WriteError {}

/// Copies `path` to a fresh `<path>.backup-<timestamp>[-n]`, then overwrites it. The original is
/// never touched unless the backup succeeded. The suffix keeps two writes in one second from
/// replacing the first backup with an already-edited copy.
pub fn backup_then_write(path: &Path, bytes: &[u8], timestamp: u64) -> Result<PathBuf, WriteError> {
    if !path.is_file() {
        log::error!("mute list write refused: voice_ban.dt does not exist");
        return Err(WriteError::Missing(path.to_path_buf()));
    }
    let backup =
        (0..64).map(|n| backup_candidate(path, timestamp, n)).find(|p| !p.exists()).ok_or(WriteError::NoBackupName)?;
    std::fs::copy(path, &backup).map_err(|e| {
        log::error!("mute list backup failed, the file was not changed: {e}");
        WriteError::BackupFailed(e)
    })?;
    dp_atomic::write_atomic(path, bytes).map_err(|e| {
        log::error!("mute list write failed after the backup was saved: {e}");
        WriteError::WriteFailed { backup: backup.clone(), source: e }
    })?;
    log::info!("mute list written ({} bytes), backup saved", bytes.len());
    Ok(backup)
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
pub fn game_running_recent() -> bool {
    cached_check(&LAST_CHECK, Instant::now(), Duration::from_secs(5), dp_game::is_running)
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    NoAccount,
    NoUserdata,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoAccount => "No Steam account found.",
            Self::NoUserdata => "This account has no Steam userdata folder.",
        })
    }
}

impl std::error::Error for PathError {}

pub fn current_path() -> Result<PathBuf, PathError> {
    let account = dp_steam::current_account().ok_or(PathError::NoAccount)?;
    let dir = account.userdata_dir.ok_or(PathError::NoUserdata)?;
    Ok(voice_ban_path(Path::new(&dir)))
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
        assert!(matches!(backup_then_write(&file, b"x", 1), Err(WriteError::Missing(_))));
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
        assert!(matches!(backup_then_write(&file, b"new", 9), Err(WriteError::NoBackupName)));
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
