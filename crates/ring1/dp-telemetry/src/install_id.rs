use std::path::Path;

const FILE: &str = "install-id";

pub fn load_or_create(dir: &Path) -> String {
    let stored = std::fs::read_to_string(dir.join(FILE)).ok();
    match stored.as_deref().map(str::trim).and_then(|id| uuid::Uuid::parse_str(id).ok()) {
        Some(id) => id.to_string(),
        None => reset(dir),
    }
}

/// A write failure still returns the fresh id: the run keeps a stable id and the next start makes another.
pub fn reset(dir: &Path) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = dp_atomic::write_atomic(&dir.join(FILE), id.as_bytes()) {
        log::warn!("could not save the analytics id: {e}");
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dp-telemetry-{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn is_created_once_and_stable_across_loads() {
        let dir = temp("stable");
        let first = load_or_create(&dir);
        assert_eq!(load_or_create(&dir), first);
        assert_eq!(uuid::Uuid::parse_str(&first).unwrap().get_version_num(), 4);
    }

    #[test]
    fn reset_replaces_it_and_the_new_one_sticks() {
        let dir = temp("reset");
        let first = load_or_create(&dir);
        let second = reset(&dir);
        assert_ne!(first, second);
        assert_eq!(load_or_create(&dir), second);
    }

    #[test]
    fn a_damaged_file_is_replaced_not_reused() {
        let dir = temp("damaged");
        std::fs::write(dir.join("install-id"), "not a uuid\n").unwrap();
        let id = load_or_create(&dir);
        assert!(uuid::Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn two_installs_get_different_ids() {
        assert_ne!(load_or_create(&temp("a")), load_or_create(&temp("b")));
    }
}
