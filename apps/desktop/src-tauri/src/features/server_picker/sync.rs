//! Keeps our block rules pointing at the relay IPs Valve currently publishes. Rules are
//! compared against the live config on every run instead of trusting a revision number, so
//! it also repairs a rule someone else edited or disabled.

use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::state::ServerPickerState;
use crate::features::jobs::{JobSpec, JobsState, Policy};
use crate::features::notifications::{self, NotificationKind};
use crate::http::Http;
use dp_firewall as firewall;
use dp_server_picker::sync::sync_blocks;

const FIRST_RUN_DELAY: Duration = Duration::from_secs(20);
const INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Never registered as a running job: a sync is a short firewall edit, not something to show in
/// the status bar. It is declared only so settings can list it and switch it off.
pub(crate) const SYNC_JOB: JobSpec = JobSpec {
    id: "server-block-sync",
    title: "Updating server blocks",
    description: "Keeps the regions you block pointed at Valve's current relay addresses. Only edits Windows Firewall rules Deadlock+ already created, and tells you when it changes one.",
    default_policy: Policy::Always,
    policy_configurable: false,
};

pub fn is_enabled(app: &AppHandle) -> bool {
    app.state::<JobsState>().registry.is_enabled(SYNC_JOB.id)
}

fn summary(descriptions: &[String]) -> String {
    format!("Valve moved relay addresses for {}. Your blocks were updated to match.", descriptions.join(", "))
}

async fn run_in_background(app: &AppHandle) {
    if !is_enabled(app) {
        return;
    }
    let outcome = match sync_blocks(&app.state::<ServerPickerState>().sync_lock, &app.state::<Http>().0).await {
        Ok(outcome) => outcome,
        Err(e) => {
            log::warn!("background block sync failed: {e}");
            return;
        }
    };
    log::debug!("block sync: {} updated, {} failed", outcome.updated.len(), outcome.failed.len());

    if !outcome.updated.is_empty() {
        notifications::push(
            app,
            NotificationKind::Servers,
            "Server blocks updated",
            summary(&outcome.updated),
            Some("/server-picker".into()),
        );
    }
    if !outcome.failed.is_empty() {
        notifications::push(
            app,
            NotificationKind::Servers,
            "Some server blocks are out of date",
            format!(
                "Deadlock+ couldn't update the blocks for {}. Open the server picker and re-apply them.",
                outcome.failed.join(", ")
            ),
            Some("/server-picker".into()),
        );
    }
}

pub fn start(app: &AppHandle) {
    if !firewall::SUPPORTED {
        return;
    }
    let wake = app.state::<ServerPickerState>().sync_now.clone();
    app.state::<JobsState>().registry.on_enabled(Box::new(move |id| {
        if id == SYNC_JOB.id {
            wake.notify_one();
        }
    }));

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_RUN_DELAY).await;
        let wake = app.state::<ServerPickerState>().sync_now.clone();
        loop {
            run_in_background(&app).await;
            let _ = tokio::time::timeout(INTERVAL, wake.notified()).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_names_refreshed_regions() {
        assert_eq!(
            summary(&["Frankfurt".into(), "Stockholm".into()]),
            "Valve moved relay addresses for Frankfurt, Stockholm. Your blocks were updated to match."
        );
    }
}
