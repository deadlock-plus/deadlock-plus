use crate::features::error::error_codes;

error_codes! {
    pub enum DiagnosticsError in "diagnostics" {
        FrameLayerLinuxOnly = "frame_layer_linux_only",
        HomeFolderNotFound = "home_folder_not_found",
        FrameLayerMissing = "frame_layer_missing",
        FrameLayerInstallFailed = "frame_layer_install_failed",
        FrameLayerRemoveFailed = "frame_layer_remove_failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;

    #[test]
    fn every_diagnostics_code_is_in_the_english_catalog() {
        assert_catalogued::<DiagnosticsError>();
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn installing_the_frame_layer_off_linux_is_refused() {
        let err = crate::features::diagnostics::frames::commands::install_layer().unwrap_err();
        assert_eq!(err.code(), "diagnostics.frame_layer_linux_only");
    }
}
