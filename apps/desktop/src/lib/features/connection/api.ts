import { command } from "$lib/core/tauri";
import type { HistoryPoint, NetworkSnapshot } from "./types";

/** `allowPrompt` says the user asked for it, so the system may ask for a password. */
export function startNetworkMonitor(allowPrompt = false) {
    return command<void>("start_network_monitor", { allowPrompt });
}

export function networkSnapshot() {
    return command<NetworkSnapshot>("network_snapshot");
}

export function networkHistory() {
    return command<HistoryPoint[]>("network_history");
}
