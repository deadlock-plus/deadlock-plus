import type { AutostartStatus } from "$lib/generated/types/AutostartStatus";
import { platform, type Platform } from "$lib/core/platform";

export { getAutostart, setAutostart } from "./api";
export type { AutostartStatus };

export function autostartTitle(p: Platform = platform): string {
    return { windows: "Start with Windows", macos: "Start with macOS", linux: "Start at login" }[p];
}

export function autostartDescription(p: Platform = platform): string {
    return {
        windows: "Launches Deadlock+ when you sign in to Windows.",
        macos: "Launches Deadlock+ when you log in to macOS.",
        linux: "Launches Deadlock+ when you log in to your desktop.",
    }[p];
}

export function autostartLine(status: AutostartStatus | null, error: string | null): string {
    if (error) return `Couldn't change it: ${error}`;
    if (!status) return "";
    return status.stale
        ? "Startup is set up for a different copy of Deadlock+. Turn it off and on to use this one."
        : "";
}
