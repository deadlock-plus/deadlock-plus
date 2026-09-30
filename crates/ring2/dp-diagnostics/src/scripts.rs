use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::rules::{self, Finding};
use crate::vpk::{Vpk, VpkError};

const TABLE_START: usize = 16;
const BLOCK_LEN: usize = 12;
const SEARCH_PATH_KEY: &[u8] = b"SearchPath\0";

pub struct ScriptSource {
    pub source: String,
    /// Best effort: the header block is compressed, so the key is not always readable.
    pub search_path: Option<String>,
}

/// A resource is `size, header version, version, table offset, block count`, then a table of
/// `tag, offset (relative to its own field), size`. The `DATA` block of a script is its plain
/// UTF-8 source.
pub fn extract_source(vjs_c: &[u8]) -> Option<ScriptSource> {
    let count = u32::from_le_bytes(vjs_c.get(12..16)?.try_into().ok()?) as usize;
    let mut data = None;
    let mut header = None;
    for i in 0..count {
        let pos = TABLE_START.checked_add(i.checked_mul(BLOCK_LEN)?)?;
        let entry = vjs_c.get(pos..pos.checked_add(BLOCK_LEN)?)?;
        let tag = &entry[..4];
        let offset = u32::from_le_bytes(entry[4..8].try_into().ok()?) as usize;
        let size = u32::from_le_bytes(entry[8..12].try_into().ok()?) as usize;
        let start = (pos + 4).checked_add(offset)?;
        let payload = vjs_c.get(start..start.checked_add(size)?)?;
        match tag {
            b"DATA" => data = Some(payload),
            b"RED2" => header = Some(payload),
            _ => {}
        }
    }
    Some(ScriptSource {
        source: String::from_utf8_lossy(data?).into_owned(),
        search_path: header.and_then(search_path),
    })
}

fn search_path(header: &[u8]) -> Option<String> {
    let start = header.windows(SEARCH_PATH_KEY.len()).position(|w| w == SEARCH_PATH_KEY)? + SEARCH_PATH_KEY.len();
    let rest = &header[start..];
    let len = rest.iter().position(|&b| b == 0)?;
    (len > 0).then(|| String::from_utf8_lossy(&rest[..len]).into_owned())
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Vpk(#[from] VpkError),
}

pub struct ScriptFindings {
    /// Path of the script inside the VPK, e.g. `panorama/scripts/foo.vjs_c`.
    pub path: String,
    pub findings: Vec<Finding>,
}

pub struct VpkScan {
    /// Scripts that were readable and scanned, whether or not they produced findings.
    pub scripts_scanned: usize,
    /// First `SearchPath` seen in any script; names the addon when nothing better is known.
    pub search_path: Option<String>,
    pub flagged: Vec<ScriptFindings>,
}

/// Scans every `.vjs_c` in a `*_dir.vpk`, reading the tree and then only those entries. Entries
/// in numbered archives (`name_000.vpk`) are read from beside the dir file; ones whose archive is
/// missing are skipped. Scripts without findings are not listed in `flagged`.
pub fn scan_vpk(path: &Path) -> Result<VpkScan, ScanError> {
    let mut dir_file = File::open(path)?;
    let mut head = [0u8; 12];
    dir_file.read_exact(&mut head)?;
    let mut prefix = vec![0u8; Vpk::prefix_len(&head)?];
    dir_file.seek(SeekFrom::Start(0))?;
    dir_file.read_exact(&mut prefix)?;
    let vpk = Vpk::parse(&prefix)?;

    let mut archives: HashMap<u16, Option<File>> = HashMap::new();
    let mut scan = VpkScan { scripts_scanned: 0, search_path: None, flagged: Vec::new() };
    for entry in vpk.entries.iter().filter(|e| e.path.ends_with(".vjs_c") || e.path.ends_with(".vts_c")) {
        let (file, range) = match vpk.embedded_range(entry) {
            Some(range) => (&mut dir_file, range),
            None => {
                let slot = archives
                    .entry(entry.archive_index)
                    .or_insert_with(|| File::open(archive_path(path, entry.archive_index)).ok());
                let start = u64::from(entry.offset);
                match slot {
                    Some(file) => (file, start..start + u64::from(entry.length)),
                    None => continue,
                }
            }
        };
        let mut bytes = prefix[entry.preload.clone()].to_vec();
        let preload_len = bytes.len();
        bytes.resize(preload_len + (range.end - range.start) as usize, 0);
        file.seek(SeekFrom::Start(range.start))?;
        if file.read_exact(&mut bytes[preload_len..]).is_err() {
            continue;
        }

        let Some(script) = extract_source(&bytes) else { continue };
        scan.scripts_scanned += 1;
        if scan.search_path.is_none() {
            scan.search_path.clone_from(&script.search_path);
        }
        let findings = rules::scan(&script.source);
        if !findings.is_empty() {
            scan.flagged.push(ScriptFindings { path: entry.path.clone(), findings });
        }
    }
    Ok(scan)
}

fn archive_path(dir_vpk: &Path, index: u16) -> PathBuf {
    let name = dir_vpk.file_name().unwrap_or_default().to_string_lossy();
    let stem = name.strip_suffix("_dir.vpk").unwrap_or(&name);
    dir_vpk.with_file_name(format!("{stem}_{index:03}.vpk"))
}

#[cfg(test)]
mod tests;
