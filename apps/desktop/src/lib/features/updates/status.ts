import type { UpdatePhase } from "./updater.svelte";

export interface UpdateView {
    phase: UpdatePhase;
    version: string | null;
    progress: number | null;
    error: string | null;
}

export function updateLine(u: UpdateView): string {
    switch (u.phase) {
        case "idle":
            return "Not checked yet.";
        case "checking":
            return "Checking...";
        case "upToDate":
            return "You're on the latest version.";
        case "available":
            return `Version ${u.version} is available.`;
        case "downloading": {
            const pct = u.progress === null ? "" : ` (${Math.round(u.progress * 100)}%)`;
            return `Downloading ${u.version}${pct}. Deadlock+ restarts when it finishes.`;
        }
        case "error":
            return `Update failed: ${u.error}`;
    }
}
