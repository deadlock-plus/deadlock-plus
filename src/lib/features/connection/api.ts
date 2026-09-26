import { invoke } from "@tauri-apps/api/core";
import type { HistoryPoint, NetworkSnapshot } from "./types";

export function startNetworkMonitor() {
    return invoke<void>("start_network_monitor");
}

export function networkSnapshot() {
    return invoke<NetworkSnapshot>("network_snapshot");
}

export function networkHistory() {
    return invoke<HistoryPoint[]>("network_history");
}
