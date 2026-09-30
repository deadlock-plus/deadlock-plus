//! Keeps our block rules pointing at the relay IPs Valve currently publishes. Rules are
//! compared against the live config on every run instead of trusting a revision number, so
//! it also repairs a rule someone else edited or disabled.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::commands::SyncOutcome;
use super::definitions::load_definitions;
use super::sdr::{fetch_server_data, ServerData};
use super::state::ServerPickerState;
use super::validate::validate_block_request;
use crate::features::jobs::{JobSpec, JobsState, Policy};
use crate::features::notifications::{self, NotificationKind};
use crate::http::Http;
use dp_firewall::{self as firewall, FirewallRuleSpec};

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

/// One spec per group id, dropping any group that fails the same validation a manual block
/// goes through. A skipped group keeps whatever rule it already has.
fn desired_specs(data: &ServerData) -> Vec<FirewallRuleSpec> {
    let mut seen = HashSet::new();
    let mut specs = Vec::new();
    for group in data.unclustered.iter().chain(data.clustered.iter()) {
        if !seen.insert(group.id.as_str()) {
            continue;
        }
        if let Err(e) = validate_block_request(&group.id, &group.description, &group.relay_ips) {
            log::warn!("not syncing {}: {e}", group.id);
            continue;
        }
        specs.push(FirewallRuleSpec {
            group_id: group.id.clone(),
            description: group.description.clone(),
            relay_ips: group.relay_ips.clone(),
        });
    }
    specs
}

fn summary(descriptions: &[String]) -> String {
    format!("Valve moved relay addresses for {}. Your blocks were updated to match.", descriptions.join(", "))
}

/// Fetches Valve's current relays straight from the source (never from the web view) and
/// corrects every stale block. Runs one at a time: the picker opening and the timer can
/// land together.
pub async fn sync_blocks(state: &ServerPickerState, http: &reqwest::Client) -> Result<SyncOutcome, String> {
    let _one_at_a_time = state.sync_lock.lock().await;
    let mut outcome = SyncOutcome::default();

    for def in load_definitions() {
        let data = fetch_server_data(http, &def).await.map_err(|e| e.to_string())?;
        let specs = desired_specs(&data);
        let names: HashMap<String, String> =
            specs.iter().map(|s| (s.group_id.clone(), s.description.clone())).collect();

        let report = tauri::async_runtime::spawn_blocking(move || firewall::refresh_stale_groups(&specs))
            .await
            .map_err(|e| e.to_string())??;

        let describe = |ids: Vec<String>| ids.into_iter().map(|id| names.get(&id).cloned().unwrap_or(id));
        outcome.updated.extend(describe(report.updated));
        outcome.failed.extend(describe(report.failed));
    }

    Ok(outcome)
}

async fn run_in_background(app: &AppHandle) {
    if !is_enabled(app) {
        return;
    }
    let outcome = match sync_blocks(&app.state::<ServerPickerState>(), &app.state::<Http>().0).await {
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
    use crate::features::server_picker::sdr::{ServerData, ServerGroup};

    fn group(id: &str, description: &str, ips: &[&str]) -> ServerGroup {
        ServerGroup {
            id: id.into(),
            description: description.into(),
            is_cluster: false,
            country_code: None,
            relay_ips: ips.iter().map(|s| s.to_string()).collect(),
            member_ids: Vec::new(),
            routing_note: None,
        }
    }

    fn data(unclustered: Vec<ServerGroup>, clustered: Vec<ServerGroup>) -> ServerData {
        ServerData { revision: "1".into(), unclustered, clustered }
    }

    #[test]
    fn desired_specs_dedupe_a_group_present_in_both_lists() {
        let fra = group("fra", "Frankfurt", &["155.133.226.1"]);
        let specs = desired_specs(&data(vec![fra.clone()], vec![fra]));
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].group_id, "fra");
    }

    #[test]
    fn desired_specs_drop_groups_that_fail_validation() {
        let good = group("fra", "Frankfurt", &["155.133.226.1"]);
        let lan = group("evil", "Evil", &["192.168.1.1"]);
        let empty = group("none", "None", &[]);
        let specs = desired_specs(&data(vec![good, lan, empty], Vec::new()));
        assert_eq!(specs.iter().map(|s| s.group_id.as_str()).collect::<Vec<_>>(), ["fra"]);
    }

    #[test]
    fn desired_specs_keep_clusters_and_pops_apart() {
        let pop = group("fra", "Frankfurt (Germany)", &["1.1.1.1"]);
        let cluster = group("frankfurt", "Frankfurt", &["1.1.1.1"]);
        let specs = desired_specs(&data(vec![pop], vec![cluster]));
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn summary_names_refreshed_regions() {
        assert_eq!(
            summary(&["Frankfurt".into(), "Stockholm".into()]),
            "Valve moved relay addresses for Frankfurt, Stockholm. Your blocks were updated to match."
        );
    }
}
