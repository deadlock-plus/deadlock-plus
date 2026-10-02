import type { GcStatus } from "$lib/generated/types/GcStatus";

export function gcStatusLine(enabled: boolean, status: GcStatus | null): string {
    if (!enabled) return "Off";
    if (!status) return "";
    if (status.accounts === 0) {
        return status.lastError ? `No usable Steam login: ${status.lastError}` : "Looking for a saved Steam login.";
    }
    if (status.lastError) return `Last pass failed: ${status.lastError}`;
    const accounts = `${status.accounts} Steam account${status.accounts === 1 ? "" : "s"}`;
    return `Using ${accounts}. ${status.delivered} submitted this session.`;
}
