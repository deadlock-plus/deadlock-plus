use crate::features::error::{error_codes, AppError};
use dp_demos::cleanup::RuleError;

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
    fn a_non_numeric_match_id_is_refused_before_touching_disk() {
        let err = crate::features::demos::commands::reveal_demo("abc".into(), false).unwrap_err();
        assert_eq!(err.code(), "demos.invalid_match_id");
    }
}
