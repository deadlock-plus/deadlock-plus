use std::path::Path;

#[derive(Debug)]
pub enum ExportError {
    NotAbsolute,
    IsFolder,
    InvalidName,
    InvalidExtension,
    Write(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAbsolute => f.write_str("the save location must be an absolute path"),
            Self::IsFolder => f.write_str("the save location is a folder"),
            Self::InvalidName => f.write_str("invalid file name"),
            Self::InvalidExtension => f.write_str("invalid file extension"),
            Self::Write(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ExportError {}

pub fn write_export(path: &Path, contents: &str) -> Result<(), ExportError> {
    if !path.is_absolute() {
        return Err(ExportError::NotAbsolute);
    }
    if path.is_dir() {
        return Err(ExportError::IsFolder);
    }
    dp_atomic::write_atomic(path, contents.as_bytes()).map_err(|e| {
        log::warn!("could not save an export: {e}");
        ExportError::Write(e)
    })
}

/// The suggested name and extension come from the web view, so they are checked before they reach the dialog. The
/// dialog itself is opened here: the web view never supplies a path, so it cannot write outside what the user picks.
pub fn validate_request(default_name: &str, extension: &str) -> Result<(), ExportError> {
    let plain_name = !default_name.is_empty()
        && !default_name.contains(['/', '\\', ':'])
        && !default_name.contains("..")
        && default_name.len() <= 100;
    if !plain_name {
        return Err(ExportError::InvalidName);
    }
    if extension.is_empty() || extension.len() > 8 || !extension.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ExportError::InvalidExtension);
    }
    Ok(())
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
        assert!(matches!(write_export(Path::new("out.json"), "x"), Err(ExportError::NotAbsolute)));
        assert!(matches!(write_export(&temp_dir("dir"), "x"), Err(ExportError::IsFolder)));
    }

    #[test]
    fn accepts_a_plain_name_and_extension() {
        assert!(validate_request("deadlock-mutes.json", "json").is_ok());
    }

    #[test]
    fn rejects_names_that_carry_a_path() {
        for name in ["", r"..\evil.txt", "a/b.txt", r"a\b.txt", "C:evil.txt", "x..y"] {
            assert!(matches!(validate_request(name, "txt"), Err(ExportError::InvalidName)), "{name}");
        }
        assert!(matches!(validate_request(&"a".repeat(101), "txt"), Err(ExportError::InvalidName)));
    }

    #[test]
    fn rejects_odd_extensions() {
        for ext in ["", "j son", "exe;", "waytoolongext", "tx.t"] {
            assert!(matches!(validate_request("out", ext), Err(ExportError::InvalidExtension)), "{ext}");
        }
    }

    #[test]
    fn a_failed_write_keeps_the_existing_file() {
        let dir = temp_dir("keep");
        let path = dir.join("out.json");
        write_export(&path, "keep").unwrap();
        std::fs::create_dir(dir.join("out.json.tmp")).unwrap();
        assert!(matches!(write_export(&path, "new"), Err(ExportError::Write(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep");
    }
}
