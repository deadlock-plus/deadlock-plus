use crate::features::error::{error_codes, AppError};
use dp_voice_bans::{looks_like_voice_ban, PathError, WriteError};

error_codes! {
    pub enum VoiceBansError in "voice_bans" {
        NoAccount = "no_account",
        NoUserdata = "no_userdata",
        NotUtf8 = "not_utf8",
        ReadFailed = "read_failed",
        GameRunning = "game_running",
        NotAMuteList = "not_a_mute_list",
        FileMissing = "file_missing",
        NoBackupName = "no_backup_name",
        BackupFailed = "backup_failed",
        WriteFailed = "write_failed",
    }
}

impl From<PathError> for AppError {
    fn from(error: PathError) -> Self {
        AppError::new(match error {
            PathError::NoAccount => VoiceBansError::NoAccount,
            PathError::NoUserdata => VoiceBansError::NoUserdata,
        })
    }
}

impl From<WriteError> for AppError {
    fn from(error: WriteError) -> Self {
        match &error {
            WriteError::Missing(path) => AppError::new(VoiceBansError::FileMissing).param("path", path.display()),
            WriteError::NoBackupName => AppError::new(VoiceBansError::NoBackupName),
            WriteError::BackupFailed(_) => AppError::new(VoiceBansError::BackupFailed).detail(&error),
            WriteError::WriteFailed { backup, .. } => {
                AppError::new(VoiceBansError::WriteFailed).param("backup", backup.display()).detail(&error)
            }
        }
    }
}

pub fn check_writable(text: &str, game_running: bool) -> Result<(), AppError> {
    if game_running {
        return Err(VoiceBansError::GameRunning.into());
    }
    if !looks_like_voice_ban(text.as_bytes()) {
        return Err(VoiceBansError::NotAMuteList.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;
    use std::path::PathBuf;

    #[test]
    fn every_voice_bans_code_is_in_the_english_catalog() {
        assert_catalogued::<VoiceBansError>();
    }

    #[test]
    fn path_errors_map_to_their_own_codes() {
        assert_eq!(AppError::from(PathError::NoAccount).code(), "voice_bans.no_account");
        assert_eq!(AppError::from(PathError::NoUserdata).code(), "voice_bans.no_userdata");
    }

    #[test]
    fn write_errors_map_to_their_own_codes() {
        let io = || std::io::Error::other("x");
        assert_eq!(AppError::from(WriteError::Missing(PathBuf::from("a"))).code(), "voice_bans.file_missing");
        assert_eq!(AppError::from(WriteError::NoBackupName).code(), "voice_bans.no_backup_name");
        assert_eq!(AppError::from(WriteError::BackupFailed(io())).code(), "voice_bans.backup_failed");
        let failed = WriteError::WriteFailed { backup: PathBuf::from("b"), source: io() };
        assert_eq!(AppError::from(failed).code(), "voice_bans.write_failed");
    }

    #[test]
    fn a_write_while_the_game_runs_is_refused_first() {
        assert_eq!(check_writable("garbage", true).unwrap_err().code(), "voice_bans.game_running");
    }

    #[test]
    fn text_that_is_not_a_mute_list_is_refused() {
        assert_eq!(check_writable("garbage", false).unwrap_err().code(), "voice_bans.not_a_mute_list");
        assert!(check_writable("{ users = [ ] }", false).is_ok());
    }
}
