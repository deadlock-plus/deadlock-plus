import { invoke } from "@tauri-apps/api/core";

import type { AutostartStatus } from "$lib/generated/types/AutostartStatus";

export type { AutostartStatus };

export function getAutostart() {
    return invoke<AutostartStatus>("autostart_status");
}

export function setAutostart(enabled: boolean) {
    return invoke<AutostartStatus>("set_autostart", { enabled });
}

export function autostartLine(status: AutostartStatus | null, error: string | null): string {
    if (error) return `Couldn't change it: ${error}`;
    if (!status) return "";
    return status.stale
        ? "Startup is set up for a different copy of Deadlock+. Turn it off and on to use this one."
        : "";
}
