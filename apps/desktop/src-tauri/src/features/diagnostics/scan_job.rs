use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use ts_rs::TS;

use super::commands::{list_addons_from, scan_addon_in, AddonListing, AddonScan};
use crate::features::jobs::{Flow, JobHandle, JobSpec, Policy};
use dp_sync::LockExt;

pub const JOB: JobSpec = JobSpec {
    id: "addon-scan",
    title: "Scanning addons",
    description:
        "Checks your installed addons for scripts that can hurt frametimes. Runs shortly after Deadlock+ opens.",
    default_policy: Policy::PauseInGame,
    policy_configurable: true,
};

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonFailure {
    pub file_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddonScanReport {
    pub listing: Option<AddonListing>,
    pub scans: Vec<AddonScan>,
    pub failures: Vec<AddonFailure>,
}

#[derive(Default)]
pub struct AddonScanState {
    report: Mutex<AddonScanReport>,
}

impl AddonScanState {
    pub fn report(&self) -> AddonScanReport {
        self.report.lock_or_recover().clone()
    }

    pub fn reset(&self) {
        *self.report.lock_or_recover() = AddonScanReport::default();
    }
}

/// Scans every addon in `dir`, checkpointing between addons so the job can pause or stop.
pub fn run(dir: Option<&Path>, handle: &JobHandle, state: &AddonScanState) {
    handle.start();
    let listing = list_addons_from(dir);
    let names: Vec<String> = listing.addons.iter().map(|a| a.file_name.clone()).collect();
    state.report.lock_or_recover().listing = Some(listing);
    let total = names.len();
    handle.progress(0, total, None);

    for (i, name) in names.iter().enumerate() {
        if handle.checkpoint() == Flow::Cancelled {
            return;
        }
        handle.progress(i, total, Some(name));
        let result =
            dir.ok_or_else(|| "Deadlock addons folder not found".to_string()).and_then(|d| scan_addon_in(d, name));
        let mut report = state.report.lock_or_recover();
        match result {
            Ok(scan) => report.scans.push(scan),
            Err(message) => {
                log::warn!("addon scan failed for {name}: {message}");
                report.failures.push(AddonFailure { file_name: name.clone(), message });
            }
        }
        drop(report);
        handle.progress(i + 1, total, Some(name));
    }
    handle.finish();
}

#[cfg(test)]
mod tests;
