use std::path::Path;

/// Opens a folder, or shows a file selected in its folder, in the system file manager.
pub fn show(path: &Path) -> Result<(), String> {
    if path.is_file() {
        tauri_plugin_opener::reveal_item_in_dir(path)
    } else {
        tauri_plugin_opener::open_path(path, None::<&str>)
    }
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_path_is_an_error_and_launches_nothing() {
        let path = std::env::temp_dir().join(format!("deadlock-plus-missing-{}", std::process::id()));
        assert!(show(&path).is_err());
    }
}
