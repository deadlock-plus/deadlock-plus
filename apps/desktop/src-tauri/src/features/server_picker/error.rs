use crate::features::error::{error_codes, AppError};
use dp_server_picker::sdr::SdrError;

error_codes! {
    pub enum ServerPickerError in "server_picker" {
        UnknownGame = "unknown_game",
        InvalidRequest = "invalid_request",
        NoRelayData = "no_relay_data",
        BlockFailed = "block_failed",
        UnblockFailed = "unblock_failed",
        ListBlockedFailed = "list_blocked_failed",
        SyncFailed = "sync_failed",
        ExternalScanFailed = "external_scan_failed",
        ExternalImportFailed = "external_import_failed",
    }
}

impl From<SdrError> for AppError {
    fn from(error: SdrError) -> Self {
        match error {
            SdrError::Request(e) => AppError::network(e),
            SdrError::Empty => AppError::new(ServerPickerError::NoRelayData),
        }
    }
}

pub fn unknown_game(game_id: &str) -> AppError {
    AppError::new(ServerPickerError::UnknownGame).detail(format!("unknown game id: {game_id}"))
}

pub fn invalid_request(reason: String) -> AppError {
    AppError::new(ServerPickerError::InvalidRequest).detail(reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;
    use crate::features::server_picker::commands;

    #[test]
    fn every_server_picker_code_is_in_the_english_catalog() {
        assert_catalogued::<ServerPickerError>();
    }

    #[test]
    fn an_empty_relay_list_maps_to_its_own_code() {
        assert_eq!(AppError::from(SdrError::Empty).code(), "server_picker.no_relay_data");
    }

    #[test]
    fn an_unknown_game_keeps_the_id_out_of_the_code() {
        assert_eq!(unknown_game("nope").code(), "server_picker.unknown_game");
    }

    #[test]
    fn blocking_a_bad_group_id_is_refused_before_touching_the_firewall() {
        let request = commands::BlockGroupRequest {
            id: "../evil".into(),
            description: "x".into(),
            relay_ips: vec!["155.133.248.1".into()],
        };
        let err = commands::block_server_groups(vec![request]).unwrap_err();
        assert_eq!(err.code(), "server_picker.invalid_request");
    }

    #[test]
    fn unblocking_a_bad_group_id_is_refused_before_touching_the_firewall() {
        let err = commands::unblock_server_groups(vec!["../evil".into()]).unwrap_err();
        assert_eq!(err.code(), "server_picker.invalid_request");
    }

    #[test]
    fn listing_a_bad_group_id_is_refused_before_touching_the_firewall() {
        let err = commands::list_blocked_group_ids(vec!["../evil".into()]).unwrap_err();
        assert_eq!(err.code(), "server_picker.invalid_request");
    }
}
