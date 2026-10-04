use crate::features::error::{error_codes, AppError};

error_codes! {
    pub enum IngestError in "ingest" {
        Rejected = "rejected",
        RequestFailed = "request_failed",
    }
}

pub fn status_error(status: u16) -> AppError {
    match status {
        400 => IngestError::Rejected.into(),
        _ => AppError::new(IngestError::RequestFailed).param("status", status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;

    #[test]
    fn every_ingest_code_is_in_the_english_catalog() {
        assert_catalogued::<IngestError>();
    }

    #[test]
    fn a_400_is_a_rejection() {
        assert_eq!(status_error(400).code(), "ingest.rejected");
    }

    #[test]
    fn other_statuses_keep_the_number_as_a_param() {
        let json = serde_json::to_value(status_error(503)).unwrap();
        assert_eq!(json, serde_json::json!({ "code": "ingest.request_failed", "params": { "status": "503" } }));
    }
}
