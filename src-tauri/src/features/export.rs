use std::path::Path;

fn write_export(path: &Path, contents: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("the save location must be an absolute path".into());
    }
    if path.is_dir() {
        return Err("the save location is a folder".into());
    }
    super::atomic::write_atomic(path, contents.as_bytes()).map_err(|e| {
        log::warn!("could not save an export: {e}");
        e.to_string()
    })
}

/// The suggested name and extension come from the web view, so they are checked before they reach the dialog. The
/// dialog itself is opened here: the web view never supplies a path, so it cannot write outside what the user picks.
fn validate_request(default_name: &str, extension: &str) -> Result<(), String> {
    let plain_name = !default_name.is_empty()
        && !default_name.contains(['/', '\\', ':'])
        && !default_name.contains("..")
        && default_name.len() <= 100;
    if !plain_name {
        return Err("invalid file name".into());
    }
    if extension.is_empty() || extension.len() > 8 || !extension.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("invalid file extension".into());
    }
    Ok(())
}

pub mod commands {
    use tauri_plugin_dialog::DialogExt;

    /// Asks the user where to save, then writes there. Returns false when the dialog is cancelled.
    #[tauri::command]
    pub async fn save_text_file(
        app: tauri::AppHandle,
        default_name: String,
        filter_name: String,
        extension: String,
        contents: String,
    ) -> Result<bool, String> {
        super::validate_request(&default_name, &extension)?;
        let picked = tauri::async_runtime::spawn_blocking(move || {
            app.dialog()
                .file()
                .set_file_name(default_name)
                .add_filter(filter_name, &[extension.as_str()])
                .blocking_save_file()
        })
        .await
        .map_err(|e| e.to_string())?;
        let Some(picked) = picked else { return Ok(false) };
        let path = picked.into_path().map_err(|e| e.to_string())?;
        super::write_export(&path, &contents)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-export-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn writes_the_contents() {
        let path = temp_dir("write").join("out.json");
        write_export(&path, "[1]").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");
    }

    #[test]
    fn refuses_relative_paths_and_folders() {
        assert!(write_export(Path::new("out.json"), "x").is_err());
        assert!(write_export(&temp_dir("dir"), "x").is_err());
    }

    #[test]
    fn accepts_a_plain_name_and_extension() {
        assert!(validate_request("deadlock-mutes.json", "json").is_ok());
    }

    #[test]
    fn rejects_names_that_carry_a_path() {
        for name in ["", r"..\evil.txt", "a/b.txt", r"a\b.txt", "C:evil.txt", "x..y"] {
            assert!(validate_request(name, "txt").is_err(), "{name}");
        }
        assert!(validate_request(&"a".repeat(101), "txt").is_err());
    }

    #[test]
    fn rejects_odd_extensions() {
        for ext in ["", "j son", "exe;", "waytoolongext", "tx.t"] {
            assert!(validate_request("out", ext).is_err(), "{ext}");
        }
    }

    #[test]
    fn a_failed_write_keeps_the_existing_file() {
        let dir = temp_dir("keep");
        let path = dir.join("out.json");
        write_export(&path, "keep").unwrap();
        std::fs::create_dir(dir.join("out.json.tmp")).unwrap();
        assert!(write_export(&path, "new").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep");
    }
}
