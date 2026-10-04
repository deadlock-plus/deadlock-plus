use crate::features::error::{error_codes, AppError};
use dp_export::ExportError as SaveError;

error_codes! {
    pub enum ExportError in "export" {
        InvalidName = "invalid_name",
        InvalidExtension = "invalid_extension",
        InvalidLocation = "invalid_location",
        SaveFailed = "save_failed",
    }
}

impl From<SaveError> for AppError {
    fn from(error: SaveError) -> Self {
        match error {
            SaveError::InvalidName => ExportError::InvalidName.into(),
            SaveError::InvalidExtension => ExportError::InvalidExtension.into(),
            SaveError::NotAbsolute | SaveError::IsFolder => ExportError::InvalidLocation.into(),
            SaveError::Write(e) => AppError::new(ExportError::SaveFailed).detail(e),
        }
    }
}

pub mod commands {
    use crate::features::error::AppError;
    use tauri_plugin_dialog::DialogExt;

    /// Asks the user where to save, then writes there. Returns false when the dialog is cancelled.
    #[tauri::command]
    pub async fn save_text_file(
        app: tauri::AppHandle,
        default_name: String,
        filter_name: String,
        extension: String,
        contents: String,
    ) -> Result<bool, AppError> {
        dp_export::validate_request(&default_name, &extension)?;
        let picked = tauri::async_runtime::spawn_blocking(move || {
            app.dialog()
                .file()
                .set_file_name(default_name)
                .add_filter(filter_name, &[extension.as_str()])
                .blocking_save_file()
        })
        .await
        .map_err(AppError::internal)?;
        let Some(picked) = picked else { return Ok(false) };
        let path = picked.into_path().map_err(AppError::io)?;
        dp_export::write_export(&path, &contents)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;

    #[test]
    fn every_export_code_is_in_the_english_catalog() {
        assert_catalogued::<ExportError>();
    }

    #[test]
    fn save_errors_map_to_their_own_codes() {
        assert_eq!(AppError::from(SaveError::InvalidName).code(), "export.invalid_name");
        assert_eq!(AppError::from(SaveError::InvalidExtension).code(), "export.invalid_extension");
        assert_eq!(AppError::from(SaveError::NotAbsolute).code(), "export.invalid_location");
        assert_eq!(AppError::from(SaveError::IsFolder).code(), "export.invalid_location");
        assert_eq!(AppError::from(SaveError::Write(std::io::Error::other("x"))).code(), "export.save_failed");
    }

    #[test]
    fn a_bad_request_maps_to_a_coded_error() {
        let err = AppError::from(dp_export::validate_request("a/b", "json").unwrap_err());
        assert_eq!(err.code(), "export.invalid_name");
    }
}
