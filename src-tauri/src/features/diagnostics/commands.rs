use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use super::addons::{addons_dir, label, list_addons_in, resolve_addon};
use super::rules::Finding;
use super::scripts::scan_vpk;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonInfo {
    pub file_name: String,
    pub mod_id: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonListing {
    pub dir: Option<String>,
    pub addons: Vec<AddonInfo>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptReport {
    pub path: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonScan {
    pub file_name: String,
    pub label: String,
    pub scripts_scanned: usize,
    pub scripts: Vec<ScriptReport>,
}

pub fn list_addons_from(dir: Option<&Path>) -> AddonListing {
    let Some(dir) = dir else {
        return AddonListing { dir: None, addons: Vec::new() };
    };
    let addons = list_addons_in(dir)
        .into_iter()
        .map(|a| AddonInfo { file_name: a.file_name, mod_id: a.mod_id, enabled: a.enabled })
        .collect();
    AddonListing { dir: Some(dir.to_string_lossy().into_owned()), addons }
}

pub fn scan_addon_in(dir: &Path, file_name: &str) -> Result<AddonScan, String> {
    let path = resolve_addon(dir, file_name).ok_or_else(|| format!("no addon named {file_name}"))?;
    let scan = scan_vpk(&path).map_err(|e| format!("could not read {file_name}: {e}"))?;
    let mod_id = list_addons_in(dir).into_iter().find(|a| a.file_name == file_name).and_then(|a| a.mod_id);
    Ok(AddonScan {
        file_name: file_name.to_string(),
        label: label(scan.search_path.as_deref(), mod_id.as_deref(), file_name),
        scripts_scanned: scan.scripts_scanned,
        scripts: scan
            .flagged
            .into_iter()
            .map(|s| ScriptReport { path: s.path, findings: s.findings })
            .collect(),
    })
}

#[tauri::command]
pub async fn list_addons() -> Result<AddonListing, String> {
    tauri::async_runtime::spawn_blocking(|| list_addons_from(addons_dir().as_deref()))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_addon(file_name: String) -> Result<AddonScan, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = addons_dir().ok_or_else(|| "Deadlock addons folder not found".to_string())?;
        let result = scan_addon_in(&dir, &file_name);
        match &result {
            Ok(scan) => log::debug!(
                "scanned {file_name}: {} scripts, {} flagged",
                scan.scripts_scanned,
                scan.scripts.len()
            ),
            Err(e) => log::warn!("addon scan failed: {e}"),
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests;
