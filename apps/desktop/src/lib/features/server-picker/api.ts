import { invoke } from "@tauri-apps/api/core";
import type { ExternalScan, FirewallCapability, GameDefinition, PingResults, ServerData, SyncOutcome } from "./types";

export function getGameDefinitions() {
    return invoke<GameDefinition[]>("get_game_definitions");
}

export function fetchServerGroups(gameId: string) {
    return invoke<ServerData>("fetch_server_groups", { gameId });
}

export function pingServerGroups(groups: { id: string; relayIps: string[] }[]) {
    return invoke<PingResults>("ping_server_groups", { groups });
}

export function blockServerGroups(groups: { id: string; description: string; relayIps: string[] }[]) {
    return invoke<void>("block_server_groups", { groups });
}

export function unblockServerGroups(ids: string[]) {
    return invoke<void>("unblock_server_groups", { ids });
}

export function listBlockedGroupIds(candidateIds: string[]) {
    return invoke<string[]>("list_blocked_group_ids", { candidateIds });
}

export function syncServerBlocks() {
    return invoke<SyncOutcome>("sync_server_blocks");
}

export function firewallCapability() {
    return invoke<FirewallCapability>("firewall_capability");
}

export function detectExternalBlocks(gameId: string) {
    return invoke<ExternalScan>("detect_external_blocks", { gameId });
}

export function importExternalBlocks(gameId: string) {
    return invoke<string[]>("import_external_blocks", { gameId });
}
