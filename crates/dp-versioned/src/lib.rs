use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::io;
use std::path::Path;

/// Upgrades the stored data one schema version. `migrations[i]` turns version `i + 1` into `i + 2`.
pub type Migration = fn(Value) -> Value;

const VERSION_KEY: &str = "schema_version";
const DATA_KEY: &str = "data";

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Corrupt(String),
    /// Written by a newer build. Never migrated down and never overwritten.
    Newer {
        found: u32,
        supported: u32,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Corrupt(e) => write!(f, "unreadable: {e}"),
            Self::Newer { found, supported } => {
                write!(f, "written by a newer version (schema {found}, this build supports {supported})")
            }
        }
    }
}

pub fn current_version(migrations: &[Migration]) -> u32 {
    1 + migrations.len() as u32
}

fn split(value: Value) -> (u32, Value) {
    match value {
        Value::Object(mut map) if map.contains_key(VERSION_KEY) && map.contains_key(DATA_KEY) => {
            let version = map.get(VERSION_KEY).and_then(Value::as_u64).unwrap_or(0) as u32;
            (version, map.remove(DATA_KEY).unwrap_or(Value::Null))
        }
        legacy => (0, legacy),
    }
}

/// `Ok(None)` when the file does not exist. A file without a version envelope is schema 0 and reads as
/// schema 1 unchanged.
pub fn read<T: DeserializeOwned>(path: &Path, migrations: &[Migration]) -> Result<Option<T>, ReadError> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ReadError::Io(e)),
    };
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| ReadError::Corrupt(e.to_string()))?;
    let (found, mut data) = split(value);
    let supported = current_version(migrations);
    if found > supported {
        return Err(ReadError::Newer { found, supported });
    }
    for migrate in &migrations[found.max(1) as usize - 1..] {
        data = migrate(data);
    }
    serde_json::from_value(data).map(Some).map_err(|e| ReadError::Corrupt(e.to_string()))
}

/// Refuses to replace a file written by a newer build, so a downgrade cannot destroy its data. An
/// unreadable existing file is replaced.
pub fn write<T: Serialize>(path: &Path, migrations: &[Migration], value: &T) -> io::Result<()> {
    let supported = current_version(migrations);
    if let Ok(existing) = std::fs::read(path) {
        if let Ok(existing) = serde_json::from_slice::<Value>(&existing) {
            let (found, _) = split(existing);
            if found > supported {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    ReadError::Newer { found, supported }.to_string(),
                ));
            }
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let envelope = json!({ VERSION_KEY: supported, DATA_KEY: value });
    dp_atomic::write_atomic(path, &serde_json::to_vec(&envelope)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-versioned-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    type Map = BTreeMap<String, u32>;

    fn rename_key(mut v: Value) -> Value {
        let map = v.as_object_mut().unwrap();
        if let Some(old) = map.remove("old") {
            map.insert("new".into(), old);
        }
        v
    }

    #[test]
    fn missing_file_reads_as_none() {
        let dir = temp_dir("missing");
        assert!(read::<Map>(&dir.join("a.json"), &[]).unwrap().is_none());
    }

    #[test]
    fn round_trips_through_the_envelope() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("a.json");
        let map = Map::from([("a".into(), 1)]);
        write(&path, &[], &map).unwrap();
        let on_disk: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(on_disk["schema_version"], 1);
        assert_eq!(read::<Map>(&path, &[]).unwrap().unwrap(), map);
    }

    #[test]
    fn legacy_file_without_an_envelope_reads_unchanged() {
        let dir = temp_dir("legacy");
        let path = dir.join("a.json");
        std::fs::write(&path, r#"{"a":1}"#).unwrap();
        assert_eq!(read::<Map>(&path, &[]).unwrap().unwrap(), Map::from([("a".into(), 1)]));
    }

    #[test]
    fn a_legacy_file_with_a_version_field_is_not_mistaken_for_an_envelope() {
        let dir = temp_dir("legacy-version");
        let path = dir.join("a.json");
        std::fs::write(&path, r#"{"version":9,"data":2}"#).unwrap();
        assert_eq!(read::<Map>(&path, &[]).unwrap().unwrap(), Map::from([("version".into(), 9), ("data".into(), 2)]));
    }

    #[test]
    fn migrations_run_in_order_from_the_stored_version() {
        let dir = temp_dir("migrate");
        let path = dir.join("a.json");
        std::fs::write(&path, r#"{"schema_version":1,"data":{"old":5}}"#).unwrap();
        assert_eq!(read::<Map>(&path, &[rename_key]).unwrap().unwrap(), Map::from([("new".into(), 5)]));
    }

    #[test]
    fn legacy_files_get_every_migration() {
        let dir = temp_dir("legacy-migrate");
        let path = dir.join("a.json");
        std::fs::write(&path, r#"{"old":5}"#).unwrap();
        assert_eq!(read::<Map>(&path, &[rename_key]).unwrap().unwrap(), Map::from([("new".into(), 5)]));
    }

    #[test]
    fn a_current_file_skips_migrations() {
        let dir = temp_dir("skip");
        let path = dir.join("a.json");
        std::fs::write(&path, r#"{"schema_version":2,"data":{"old":5}}"#).unwrap();
        assert_eq!(read::<Map>(&path, &[rename_key]).unwrap().unwrap(), Map::from([("old".into(), 5)]));
    }

    #[test]
    fn a_newer_file_is_refused_on_read_and_survives_a_write() {
        let dir = temp_dir("newer");
        let path = dir.join("a.json");
        let original = r#"{"schema_version":7,"data":{"future":1}}"#;
        std::fs::write(&path, original).unwrap();
        assert!(matches!(read::<Map>(&path, &[]), Err(ReadError::Newer { found: 7, supported: 1 })));
        assert!(write(&path, &[], &Map::new()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn corrupt_files_are_reported_and_can_be_replaced() {
        let dir = temp_dir("corrupt");
        let path = dir.join("a.json");
        std::fs::write(&path, "{ nope").unwrap();
        assert!(matches!(read::<Map>(&path, &[]), Err(ReadError::Corrupt(_))));
        write(&path, &[], &Map::from([("a".into(), 1)])).unwrap();
        assert!(read::<Map>(&path, &[]).unwrap().is_some());
    }

    #[test]
    fn write_creates_the_parent_folder() {
        let dir = temp_dir("mkdir");
        let path = dir.join("nested").join("a.json");
        write(&path, &[], &Map::new()).unwrap();
        assert!(path.exists());
    }
}
