import { invoke } from "@tauri-apps/api/core";
import type { HistoryPoint, NetworkSnapshot } from "./types";

/** `allowPrompt` says the user asked for it, so the system may ask for a password. */
export function startNetworkMonitor(allowPrompt = false) {
    return invoke<void>("start_network_monitor", { allowPrompt });
}

export function networkSnapshot() {
    return invoke<NetworkSnapshot>("network_snapshot");
}

export function networkHistory() {
    return invoke<HistoryPoint[]>("network_history");
}
