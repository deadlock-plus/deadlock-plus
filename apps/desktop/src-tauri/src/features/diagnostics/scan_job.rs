use std::path::Path;
use std::sync::Mutex;

use dp_diagnostics::scan::{self, scan_all, AddonFailure, AddonListing, AddonScan, AddonScanReport, ScanObserver};

use crate::features::jobs::{Flow, JobHandle, JobSpec, Policy};
use dp_sync::LockExt;

pub const JOB: JobSpec = JobSpec { id: "addon-scan", default_policy: Policy::PauseInGame, policy_configurable: true };

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

struct JobObserver<'a> {
    handle: &'a JobHandle,
    state: &'a AddonScanState,
}

impl ScanObserver for JobObserver<'_> {
    fn listed(&mut self, listing: &AddonListing) {
        self.state.report.lock_or_recover().listing = Some(listing.clone());
    }

    fn progress(&mut self, done: usize, total: usize, label: Option<&str>) {
        self.handle.progress(done, total, label);
    }

    fn checkpoint(&mut self) -> scan::Flow {
        match self.handle.checkpoint() {
            Flow::Continue => scan::Flow::Continue,
            Flow::Cancelled => scan::Flow::Cancelled,
        }
    }

    fn scanned(&mut self, scan: AddonScan) {
        self.state.report.lock_or_recover().scans.push(scan);
    }

    fn failed(&mut self, failure: AddonFailure) {
        self.state.report.lock_or_recover().failures.push(failure);
    }
}

/// Scans every addon in `dir`, checkpointing between addons so the job can pause or stop.
pub fn run(dir: Option<&Path>, handle: &JobHandle, state: &AddonScanState) {
    handle.start();
    if scan_all(dir, &mut JobObserver { handle, state }) == scan::Flow::Continue {
        handle.finish();
    }
}

#[cfg(test)]
mod tests;
