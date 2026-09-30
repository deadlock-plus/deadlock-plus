import type { AutostartStatus } from "$lib/generated/types/AutostartStatus";

export { getAutostart, setAutostart } from "./api";
export type { AutostartStatus };

export function autostartLine(status: AutostartStatus | null, error: string | null): string {
    if (error) return `Couldn't change it: ${error}`;
    if (!status) return "";
    return status.stale
        ? "Startup is set up for a different copy of Deadlock+. Turn it off and on to use this one."
        : "";
}
