import { command } from "$lib/core/tauri";
import type { ExternalScan, FirewallCapability, GameDefinition, PingResults, ServerData, SyncOutcome } from "./types";

export function getGameDefinitions() {
    return command<GameDefinition[]>("get_game_definitions");
}

export function fetchServerGroups(gameId: string) {
    return command<ServerData>("fetch_server_groups", { gameId });
}

export function pingServerGroups(groups: { id: string; relayIps: string[] }[]) {
    return command<PingResults>("ping_server_groups", { groups });
}

export function blockServerGroups(groups: { id: string; description: string; relayIps: string[] }[]) {
    return command<void>("block_server_groups", { groups });
}

export function unblockServerGroups(ids: string[]) {
    return command<void>("unblock_server_groups", { ids });
}

export function listBlockedGroupIds(candidateIds: string[]) {
    return command<string[]>("list_blocked_group_ids", { candidateIds });
}

export function syncServerBlocks() {
    return command<SyncOutcome>("sync_server_blocks");
}

export function firewallCapability() {
    return command<FirewallCapability>("firewall_capability");
}

export function detectExternalBlocks(gameId: string) {
    return command<ExternalScan>("detect_external_blocks", { gameId });
}

export function importExternalBlocks(gameId: string) {
    return command<string[]>("import_external_blocks", { gameId });
}
