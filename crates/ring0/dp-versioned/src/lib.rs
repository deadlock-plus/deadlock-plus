use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom};
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

#[derive(Deserialize)]
struct StreamedEnvelope<T> {
    data: T,
    schema_version: u32,
}

#[derive(Serialize)]
struct EnvelopeRef<'a, T> {
    // Declaration order is the on-disk key order: `data` first, then the version.
    data: &'a T,
    schema_version: u32,
}

const PEEK_LEN: usize = 64;
const HEAD_PREFIX: &[u8] = br#"{"schema_version":"#;
const DATA_PREFIX: &[u8] = br#"{"data":"#;
const TAIL_KEY: &[u8] = br#","schema_version":"#;

fn parse_digits(bytes: &[u8]) -> Option<(u32, &[u8])> {
    let end = bytes.iter().position(|b| !b.is_ascii_digit()).unwrap_or(bytes.len());
    let digits = std::str::from_utf8(&bytes[..end]).ok()?;
    Some((digits.parse().ok()?, &bytes[end..]))
}

/// Reads the version of a file this crate wrote from its first and last bytes only, never parsing the
/// data. `None` means the layout was not recognised (legacy or hand-edited file). Our own writer is
/// compact, and an unescaped `"schema_version":` cannot occur inside a JSON string, so the match at
/// either end can only be the envelope key.
fn peek_version(file: &mut File) -> io::Result<Option<u32>> {
    let len = file.metadata()?.len();
    let mut head = [0u8; PEEK_LEN];
    let mut filled = 0;
    while filled < PEEK_LEN {
        match file.read(&mut head[filled..])? {
            0 => break,
            n => filled += n,
        }
    }
    let head = &head[..filled];
    if let Some(rest) = head.strip_prefix(HEAD_PREFIX) {
        return Ok(parse_digits(rest).and_then(|(v, rest)| rest.starts_with(br#","data":"#).then_some(v)));
    }
    if !head.starts_with(DATA_PREFIX) {
        return Ok(None);
    }
    let tail_len = (len as usize).min(PEEK_LEN);
    let mut tail = vec![0u8; tail_len];
    file.seek(SeekFrom::Start(len - tail_len as u64))?;
    file.read_exact(&mut tail)?;
    let tail = tail.trim_ascii_end();
    let Some(at) = tail.windows(TAIL_KEY.len()).rposition(|w| w == TAIL_KEY) else { return Ok(None) };
    Ok(parse_digits(&tail[at + TAIL_KEY.len()..]).and_then(|(v, rest)| (rest == b"}").then_some(v)))
}

/// The stored version, or 0 for a legacy file. `Ok(None)` when the file does not exist. Falls back to a
/// full parse only when the header is not in the layout this crate writes.
fn stored_version(path: &Path) -> Result<Option<u32>, ReadError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ReadError::Io(e)),
    };
    if let Some(version) = peek_version(&mut file).map_err(ReadError::Io)? {
        return Ok(Some(version));
    }
    file.rewind().map_err(ReadError::Io)?;
    let value: Value = serde_json::from_reader(BufReader::new(file)).map_err(|e| ReadError::Corrupt(e.to_string()))?;
    Ok(Some(split(value).0))
}

fn read_migrating<T: DeserializeOwned>(path: &Path, migrations: &[Migration]) -> Result<T, ReadError> {
    let file = File::open(path).map_err(ReadError::Io)?;
    let value: Value = serde_json::from_reader(BufReader::new(file)).map_err(|e| ReadError::Corrupt(e.to_string()))?;
    let (found, mut data) = split(value);
    for migrate in &migrations[found.max(1) as usize - 1..] {
        data = migrate(data);
    }
    serde_json::from_value(data).map_err(|e| ReadError::Corrupt(e.to_string()))
}

/// `Ok(None)` when the file does not exist. A file without a version envelope is schema 0 and reads as
/// schema 1 unchanged.
pub fn read<T: DeserializeOwned>(path: &Path, migrations: &[Migration]) -> Result<Option<T>, ReadError> {
    let supported = current_version(migrations);
    let Some(found) = stored_version(path)? else { return Ok(None) };
    if found > supported {
        return Err(ReadError::Newer { found, supported });
    }
    if found == supported {
        let file = File::open(path).map_err(ReadError::Io)?;
        if let Ok(envelope) = serde_json::from_reader::<_, StreamedEnvelope<T>>(BufReader::new(file)) {
            if envelope.schema_version == supported {
                return Ok(Some(envelope.data));
            }
        }
    }
    read_migrating(path, migrations).map(Some)
}

/// Refuses to replace a file written by a newer build, so a downgrade cannot destroy its data. An
/// unreadable existing file is replaced.
pub fn write<T: Serialize>(path: &Path, migrations: &[Migration], value: &T) -> io::Result<()> {
    let supported = current_version(migrations);
    if let Ok(Some(found)) = stored_version(path) {
        if found > supported {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                ReadError::Newer { found, supported }.to_string(),
            ));
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut bytes = Vec::new();
    serde_json::to_writer(&mut BufWriter::new(&mut bytes), &EnvelopeRef { data: value, schema_version: supported })?;
    dp_atomic::write_atomic(path, &bytes)
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

    #[test]
    fn a_newer_file_is_refused_without_parsing_its_data() {
        let dir = temp_dir("newer-unparsed");
        let tail_version = dir.join("tail.json");
        let head_version = dir.join("head.json");
        std::fs::write(&tail_version, r#"{"data":{"future": [1, 2, tru,"schema_version":7}"#).unwrap();
        std::fs::write(&head_version, r#"{"schema_version":7,"data":{"future": [1, 2, tru}"#).unwrap();
        for path in [&tail_version, &head_version] {
            assert!(matches!(read::<Map>(path, &[]), Err(ReadError::Newer { found: 7, supported: 1 })));
            let before = std::fs::read(path).unwrap();
            assert!(write(path, &[], &Map::new()).is_err());
            assert_eq!(std::fs::read(path).unwrap(), before);
        }
    }

    #[test]
    fn envelope_text_inside_a_string_is_not_read_as_the_version() {
        let dir = temp_dir("spoof");
        let path = dir.join("a.json");
        let map = BTreeMap::from([("x".to_string(), r#","schema_version":9}"#.to_string())]);
        write(&path, &[], &map).unwrap();
        assert_eq!(read::<BTreeMap<String, String>>(&path, &[]).unwrap().unwrap(), map);
        write(&path, &[], &map).unwrap();
    }

    #[test]
    fn the_file_layout_is_data_then_schema_version_with_no_whitespace() {
        let dir = temp_dir("layout");
        let path = dir.join("a.json");
        write(&path, &[rename_key], &Map::from([("a".into(), 1)])).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), r#"{"data":{"a":1},"schema_version":2}"#);
    }

    #[test]
    fn a_write_leaves_no_temporary_file_behind() {
        let dir = temp_dir("no-tmp");
        let path = dir.join("a.json");
        write(&path, &[], &Map::from([("a".into(), 1)])).unwrap();
        let names: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("a.json")]);
    }

    #[test]
    fn f32_values_survive_a_round_trip_exactly() {
        let dir = temp_dir("f32");
        let path = dir.join("a.json");
        let values: Vec<f32> = vec![0.1, -0.33333334, 1e-7, 123456.79];
        write(&path, &[], &values).unwrap();
        assert_eq!(read::<Vec<f32>>(&path, &[]).unwrap().unwrap(), values);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            r#"{"data":[0.1,-0.33333334,1e-7,123456.79],"schema_version":1}"#
        );
    }
}
