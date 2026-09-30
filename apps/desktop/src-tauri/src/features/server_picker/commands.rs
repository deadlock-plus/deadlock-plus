use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use ts_rs::TS;

use super::definitions::{find_definition, load_definitions, GameDefinition};
use super::ping::ping_group;
use super::sdr::{fetch_server_data, ServerData};
use super::state::ServerPickerState;
use crate::http::Http;

use super::external;
use dp_firewall as firewall;

#[tauri::command]
pub fn get_game_definitions() -> Vec<GameDefinition> {
    load_definitions()
}

#[tauri::command]
pub async fn fetch_server_groups(http: State<'_, Http>, game_id: String) -> Result<ServerData, String> {
    let def = find_definition(&game_id).ok_or_else(|| format!("unknown game id: {game_id}"))?;
    fetch_server_data(&http.0, &def).await.map_err(|e| {
        log::warn!("fetching server groups failed: {e}");
        e.to_string()
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingGroupRequest {
    pub id: String,
    pub relay_ips: Vec<String>,
}

#[tauri::command]
pub async fn ping_server_groups(groups: Vec<PingGroupRequest>) -> HashMap<String, Option<u32>> {
    let futures = groups.into_iter().map(|g| async move {
        let ms = ping_group(&g.relay_ips).await;
        (g.id, ms)
    });
    let results: HashMap<String, Option<u32>> = futures::future::join_all(futures).await.into_iter().collect();
    log::debug!("pinged {} group(s), {} answered", results.len(), results.values().filter(|ms| ms.is_some()).count());
    results
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockGroupRequest {
    pub id: String,
    pub description: String,
    pub relay_ips: Vec<String>,
}

#[tauri::command]
pub fn block_server_groups(groups: Vec<BlockGroupRequest>) -> Result<(), String> {
    for g in &groups {
        super::validate::validate_block_request(&g.id, &g.description, &g.relay_ips)?;
    }
    log::info!(
        "blocking {} server group(s): {}",
        groups.len(),
        groups.iter().map(|g| g.id.as_str()).collect::<Vec<_>>().join(", ")
    );
    let specs: Vec<firewall::FirewallRuleSpec> = groups
        .into_iter()
        .map(|g| firewall::FirewallRuleSpec { group_id: g.id, description: g.description, relay_ips: g.relay_ips })
        .collect();
    firewall::block_groups(&specs).inspect_err(|e| log::error!("blocking server groups failed: {e}"))
}

#[tauri::command]
pub fn unblock_server_groups(ids: Vec<String>) -> Result<(), String> {
    ids.iter().try_for_each(|id| super::validate::validate_group_id(id))?;
    log::info!("unblocking {} server group(s): {}", ids.len(), ids.join(", "));
    firewall::unblock_groups(&ids).inspect_err(|e| log::error!("unblocking server groups failed: {e}"))
}

/// Region descriptions, not ids, so the UI can name them directly.
#[derive(Debug, Default, Serialize, TS)]
#[ts(export)]
pub struct SyncOutcome {
    pub updated: Vec<String>,
    pub failed: Vec<String>,
}

#[tauri::command]
pub async fn sync_server_blocks(
    app: tauri::AppHandle,
    state: State<'_, ServerPickerState>,
    http: State<'_, Http>,
) -> Result<SyncOutcome, String> {
    if !firewall::SUPPORTED || !super::sync::is_enabled(&app) {
        return Ok(SyncOutcome::default());
    }
    let outcome = super::sync::sync_blocks(&state, &http.0)
        .await
        .inspect_err(|e| log::warn!("syncing server blocks failed: {e}"))?;
    if !outcome.updated.is_empty() {
        log::info!("updated {} stale block(s): {}", outcome.updated.len(), outcome.updated.join(", "));
    }
    Ok(outcome)
}

#[tauri::command]
pub fn list_blocked_group_ids(candidate_ids: Vec<String>) -> Result<Vec<String>, String> {
    candidate_ids.iter().try_for_each(|id| super::validate::validate_group_id(id))?;
    firewall::list_blocked(&candidate_ids).inspect_err(|e| log::error!("reading blocked server groups failed: {e}"))
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct FirewallCapability {
    pub supported: bool,
}

#[tauri::command]
pub fn firewall_capability() -> FirewallCapability {
    FirewallCapability { supported: firewall::SUPPORTED }
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExternalScan {
    pub rule_names: Vec<String>,
    pub sources: Vec<String>,
    pub covered_group_ids: Vec<String>,
}

#[tauri::command]
pub async fn detect_external_blocks(http: State<'_, Http>, game_id: String) -> Result<ExternalScan, String> {
    let def = find_definition(&game_id).ok_or_else(|| format!("unknown game id: {game_id}"))?;
    let data = fetch_server_data(&http.0, &def).await.map_err(|e| {
        log::warn!("fetching server groups for the external block scan failed: {e}");
        e.to_string()
    })?;
    let scan = external::scan(&data, &def).inspect_err(|e| log::error!("external block scan failed: {e}"))?;
    log::debug!("external block scan: {} rule(s), {} covered group(s)", scan.rules.len(), scan.covered_group_ids.len());
    Ok(ExternalScan {
        rule_names: scan.rules.iter().map(|r| r.name.clone()).collect(),
        sources: scan.sources,
        covered_group_ids: scan.covered_group_ids,
    })
}

#[tauri::command]
pub async fn import_external_blocks(http: State<'_, Http>, game_id: String) -> Result<Vec<String>, String> {
    let def = find_definition(&game_id).ok_or_else(|| format!("unknown game id: {game_id}"))?;
    let data = fetch_server_data(&http.0, &def).await.map_err(|e| {
        log::warn!("fetching server groups for the external block import failed: {e}");
        e.to_string()
    })?;
    let scan = external::scan(&data, &def).inspect_err(|e| log::error!("external block scan failed: {e}"))?;
    log::info!(
        "importing external blocks from {} rule(s) covering {} group(s)",
        scan.rules.len(),
        scan.covered_group_ids.len()
    );
    external::import(&data, &scan).inspect_err(|e| log::error!("importing external blocks failed: {e}"))
}
