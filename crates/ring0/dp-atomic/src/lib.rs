use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn temp_sibling(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Replaces `path` with `bytes` so a crash or full disk never leaves a half-written file: the data goes
/// to a sibling first, is flushed to disk, then renamed over the target. The parent folder must exist.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = temp_sibling(path);
    let result = File::create(&tmp)
        .and_then(|mut f| f.write_all(bytes).and_then(|()| f.sync_all()))
        .and_then(|()| fs::rename(&tmp, path));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-atomic-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn creates_a_new_file() {
        let dir = temp_dir("create");
        let path = dir.join("a.json");
        write_atomic(&path, b"one").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"one");
    }

    #[test]
    fn replaces_an_existing_file_and_leaves_no_temp_file() {
        let dir = temp_dir("replace");
        let path = dir.join("a.json");
        fs::write(&path, b"old and longer").unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(!dir.join("a.json.tmp").exists());
    }

    #[test]
    fn a_failed_write_keeps_the_original() {
        let dir = temp_dir("fail");
        let path = dir.join("a.json");
        fs::write(&path, b"keep").unwrap();
        fs::create_dir(dir.join("a.json.tmp")).unwrap();
        assert!(write_atomic(&path, b"new").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"keep");
    }

    #[test]
    fn a_failed_rename_cleans_up_the_temp_file() {
        let dir = temp_dir("rename");
        let path = dir.join("target");
        fs::create_dir(&path).unwrap();
        assert!(write_atomic(&path, b"x").is_err());
        assert!(!dir.join("target.tmp").exists());
    }

    #[test]
    fn a_missing_parent_folder_is_an_error() {
        let dir = temp_dir("noparent");
        assert!(write_atomic(&dir.join("missing").join("a.json"), b"x").is_err());
    }
}
