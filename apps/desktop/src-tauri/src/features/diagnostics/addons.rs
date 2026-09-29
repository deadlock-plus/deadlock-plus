//! Finds installed addon VPKs and names them from the Mod Manager's bookkeeping file.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const DMM_FILE: &str = ".dmm.json";
const DIR_SUFFIX: &str = "_dir.vpk";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmmMod {
    pub id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonEntry {
    pub file_name: String,
    pub mod_id: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
struct DmmFile {
    #[serde(default)]
    mods: HashMap<String, DmmRecord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DmmRecord {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    current_vpks: Vec<String>,
    #[serde(default)]
    disabled_vpks: Vec<String>,
}

/// Maps a VPK file name to the Mod Manager record that owns it. Unreadable input maps nothing.
pub fn parse_dmm(text: &str) -> HashMap<String, DmmMod> {
    let Ok(file) = serde_json::from_str::<DmmFile>(text) else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    for (id, record) in file.mods {
        for vpk in record.current_vpks {
            map.insert(vpk, DmmMod { id: id.clone(), enabled: record.enabled });
        }
        for vpk in record.disabled_vpks {
            map.insert(vpk, DmmMod { id: id.clone(), enabled: false });
        }
    }
    map
}

pub fn addons_dir() -> Option<PathBuf> {
    let dir = crate::features::storage::game_install_dir()?.join("game").join("citadel").join("addons");
    dir.is_dir().then_some(dir)
}

/// `*_dir.vpk` files in filename order, which is the game's load order (lowest wins).
pub fn list_addons_in(dir: &Path) -> Vec<AddonEntry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let dmm = std::fs::read_to_string(dir.join(DMM_FILE)).map(|t| parse_dmm(&t)).unwrap_or_default();
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(DIR_SUFFIX))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|file_name| {
            let record = dmm.get(&file_name);
            AddonEntry { mod_id: record.map(|r| r.id.clone()), enabled: record.map(|r| r.enabled), file_name }
        })
        .collect()
}

/// The path of an existing `*_dir.vpk` directly inside `dir`. Anything with a path separator or
/// parent reference is rejected, since the name comes from the frontend.
pub fn resolve_addon(dir: &Path, file_name: &str) -> Option<PathBuf> {
    let plain = !file_name.contains(['/', '\\']) && file_name != ".." && file_name != ".";
    if !plain || !file_name.ends_with(DIR_SUFFIX) {
        return None;
    }
    let path = dir.join(file_name);
    path.is_file().then_some(path)
}

pub fn label(search_path: Option<&str>, mod_id: Option<&str>, file_name: &str) -> String {
    if let Some(name) = search_path.and_then(|p| p.rsplit('/').next()).filter(|n| !n.is_empty()) {
        return name.to_string();
    }
    match mod_id {
        Some(id) => format!("GameBanana mod {id}"),
        None => file_name.to_string(),
    }
}

#[cfg(test)]
mod tests;
