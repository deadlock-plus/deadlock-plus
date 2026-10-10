use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Folder under the app data directory that holds the provisional post-game detail, one `<match id>.json` each.
pub const POSTGAME_DIR: &str = "postgame";

fn entry_path(dir: &Path, match_id: u64) -> PathBuf {
    dir.join(format!("{match_id}.json"))
}

pub fn write(dir: &Path, match_id: u64, bytes: &[u8]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    dp_atomic::write_atomic(&entry_path(dir, match_id), bytes)
}

pub fn read(dir: &Path, match_id: u64) -> io::Result<Option<Vec<u8>>> {
    match fs::read(entry_path(dir, match_id)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Returns whether a file was there.
pub fn remove(dir: &Path, match_id: u64) -> io::Result<bool> {
    match fs::remove_file(entry_path(dir, match_id)) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-postgame-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_written_entry_reads_back_and_a_missing_one_is_none() {
        let dir = temp_dir("round").join("postgame");
        write(&dir, 123, b"{\"a\":1}").unwrap();
        assert_eq!(read(&dir, 123).unwrap(), Some(b"{\"a\":1}".to_vec()));
        assert_eq!(read(&dir, 456).unwrap(), None);
        assert!(dir.join("123.json").is_file());
    }

    #[test]
    fn writing_again_replaces_the_entry() {
        let dir = temp_dir("replace");
        write(&dir, 1, b"first").unwrap();
        write(&dir, 1, b"second").unwrap();
        assert_eq!(read(&dir, 1).unwrap(), Some(b"second".to_vec()));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn remove_deletes_the_entry_and_reports_whether_it_existed() {
        let dir = temp_dir("remove");
        write(&dir, 1, b"x").unwrap();
        assert!(remove(&dir, 1).unwrap());
        assert!(!remove(&dir, 1).unwrap());
        assert!(!remove(&temp_dir("absent").join("nope"), 9).unwrap());
        assert_eq!(read(&dir, 1).unwrap(), None);
    }
}
