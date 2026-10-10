import { command } from "$lib/core/tauri";
import type { HistoryPoint, NetworkPoll, NetworkSnapshot, PingSummary } from "./types";

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

/** Direct ping over `[startMs, endMs]`, read from the full on-disk log. */
export function networkHistoryRange(startMs: number, endMs: number) {
    return command<PingSummary | null>("network_history_range", { startMs, endMs });
}

/** Raw pings over `[startMs, endMs]`, thinned on the Rust side to at most `maxPoints`. */
export function networkHistoryPoints(startMs: number, endMs: number, maxPoints: number) {
    return command<HistoryPoint[]>("network_history_points", { startMs, endMs, maxPoints });
}

/** Current snapshot plus the history points newer than `sinceT`, in one call. */
export function networkPoll(sinceT: number | null) {
    return command<NetworkPoll>("network_poll", { sinceT });
}
