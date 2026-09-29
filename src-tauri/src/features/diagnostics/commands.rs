use std::path::Path;

use serde::Serialize;
use ts_rs::TS;

use super::addons::{addons_dir, label, list_addons_in, resolve_addon};
use super::rules::Finding;
use super::scan_job::{self, AddonScanReport, AddonScanState};
use super::scripts::scan_vpk;
use crate::features::jobs::JobsState;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonInfo {
    pub file_name: String,
    pub mod_id: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonListing {
    pub dir: Option<String>,
    pub addons: Vec<AddonInfo>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptReport {
    pub path: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, TS)]
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
        scripts: scan.flagged.into_iter().map(|s| ScriptReport { path: s.path, findings: s.findings }).collect(),
    })
}

#[tauri::command]
pub async fn start_addon_scan(app: tauri::AppHandle, force: bool) -> Result<(), String> {
    let jobs = app.state::<JobsState>();
    if jobs.registry.is_active(scan_job::JOB.id) {
        if force {
            jobs.registry.force_run(scan_job::JOB.id);
        }
        return Ok(());
    }
    app.state::<AddonScanState>().reset();
    let handle = jobs.registry.register(scan_job::JOB);
    if force {
        jobs.registry.force_run(scan_job::JOB.id);
    }
    let dir = addons_dir();
    tauri::async_runtime::spawn_blocking(move || {
        scan_job::run(dir.as_deref(), &handle, &app.state::<AddonScanState>());
    });
    Ok(())
}

#[tauri::command]
pub async fn addon_scan_report(state: tauri::State<'_, AddonScanState>) -> Result<AddonScanReport, ()> {
    Ok(state.report())
}

#[cfg(test)]
mod tests;
