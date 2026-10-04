use crate::features::error::{error_codes, AppError};
use dp_demos::cleanup::RuleError;
use dp_demos::delete::FailureReason;

error_codes! {
    pub enum DemosError in "demos" {
        ReplaysFolderNotFound = "replays_folder_not_found",
        InvalidMatchId = "invalid_match_id",
        ReplayMissing = "replay_missing",
        RecycleBinFull = "recycle_bin_full",
        RevealFailed = "reveal_failed",
        PinSaveFailed = "pin_save_failed",
        RulesSaveFailed = "rules_save_failed",
        RulesDuplicateId = "rules_duplicate_id",
        RulesZeroDays = "rules_zero_days",
        RulesZeroSize = "rules_zero_size",
        NotAReplay = "not_a_replay",
        ReplayPinned = "replay_pinned",
        DeleteFailed = "delete_failed",
    }
}

impl From<RuleError> for AppError {
    fn from(error: RuleError) -> Self {
        AppError::new(match error {
            RuleError::DuplicateId => DemosError::RulesDuplicateId,
            RuleError::ZeroDays => DemosError::RulesZeroDays,
            RuleError::ZeroSize => DemosError::RulesZeroSize,
        })
    }
}

impl From<FailureReason> for AppError {
    fn from(reason: FailureReason) -> Self {
        AppError::new(match reason {
            FailureReason::NotAReplay => DemosError::NotAReplay,
            FailureReason::Missing => DemosError::ReplayMissing,
            FailureReason::Pinned => DemosError::ReplayPinned,
            FailureReason::Io => DemosError::DeleteFailed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::{assert_catalogued, AppError};
    use dp_demos::cleanup::RuleError;

    #[test]
    fn every_demos_code_is_in_the_english_catalog() {
        assert_catalogued::<DemosError>();
    }

    #[test]
    fn rule_errors_map_to_their_own_codes() {
        assert_eq!(AppError::from(RuleError::DuplicateId).code(), "demos.rules_duplicate_id");
        assert_eq!(AppError::from(RuleError::ZeroDays).code(), "demos.rules_zero_days");
        assert_eq!(AppError::from(RuleError::ZeroSize).code(), "demos.rules_zero_size");
    }

    #[test]
    fn delete_failure_reasons_map_to_their_own_codes() {
        assert_eq!(AppError::from(FailureReason::NotAReplay).code(), "demos.not_a_replay");
        assert_eq!(AppError::from(FailureReason::Missing).code(), "demos.replay_missing");
        assert_eq!(AppError::from(FailureReason::Pinned).code(), "demos.replay_pinned");
        assert_eq!(AppError::from(FailureReason::Io).code(), "demos.delete_failed");
    }

    #[test]
    fn a_non_numeric_match_id_is_refused_before_touching_disk() {
        let err = crate::features::demos::commands::reveal_demo("abc".into(), false).unwrap_err();
        assert_eq!(err.code(), "demos.invalid_match_id");
    }
}
