use crate::features::error::{error_codes, AppError};
use dp_storage::{is_clearable, EntryId};

error_codes! {
    pub enum StorageError in "storage" {
        LocationMissing = "location_missing",
        NotClearable = "not_clearable",
        GameRunning = "game_running",
        RevealFailed = "reveal_failed",
    }
}

pub fn check_clear(id: EntryId, game_running: bool) -> Result<(), AppError> {
    if !is_clearable(id) {
        return Err(StorageError::NotClearable.into());
    }
    if game_running {
        return Err(StorageError::GameRunning.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;

    #[test]
    fn every_storage_code_is_in_the_english_catalog() {
        assert_catalogued::<StorageError>();
    }

    #[test]
    fn an_entry_that_cannot_be_cleared_is_refused() {
        let id = dp_storage::ALL_ENTRIES.into_iter().find(|id| !is_clearable(*id)).expect("a read-only entry exists");
        assert_eq!(check_clear(id, false).unwrap_err().code(), "storage.not_clearable");
    }

    #[test]
    fn clearing_is_refused_while_the_game_runs() {
        assert_eq!(check_clear(EntryId::ShaderCache, true).unwrap_err().code(), "storage.game_running");
        assert!(check_clear(EntryId::ShaderCache, false).is_ok());
    }
}
