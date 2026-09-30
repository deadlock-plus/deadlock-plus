import type { DemoStatus } from "$lib/generated/types/DemoStatus";
import type { Demo } from "$lib/generated/types/Demo";
import type { DemoListing } from "$lib/generated/types/DemoListing";
import type { PlayerSummary } from "$lib/generated/types/PlayerSummary";
import type { MatchSummary } from "$lib/generated/types/MatchSummary";
import type { MetaResult } from "$lib/generated/types/MetaResult";
import type { RecycleAvailability } from "$lib/generated/types/RecycleAvailability";
import type { DeleteMode } from "$lib/generated/types/DeleteMode";
import type { DeletePreview } from "$lib/generated/types/DeletePreview";
import type { DeleteReport } from "$lib/generated/types/DeleteReport";
import type { CleanupRule } from "$lib/generated/types/CleanupRule";
import type { CleanupMatch } from "$lib/generated/types/CleanupMatch";

export type {
    DemoStatus,
    Demo,
    DemoListing,
    PlayerSummary,
    MatchSummary,
    MetaResult,
    RecycleAvailability,
    DeleteMode,
    DeletePreview,
    DeleteReport,
    CleanupRule,
    CleanupMatch,
};

export function statlockerMatchUrl(matchId: number): string {
    return `https://statlocker.gg/match/${matchId}/summary`;
}

/** `accountIds` is ordered by preference; the first account found in the match wins. */
export function myPlayer(summary: MatchSummary, accountIds: number[]): PlayerSummary | null {
    for (const id of accountIds) {
        const p = summary.players.find((x) => x.accountId === id);
        if (p) return p;
    }
    return null;
}

export function matchResult(summary: MatchSummary, accountIds: number[]): "win" | "loss" | null {
    const me = myPlayer(summary, accountIds);
    if (!me) return null;
    return me.team === summary.winningTeam ? "win" : "loss";
}

export function formatDuration(totalSeconds: number): string {
    const h = Math.floor(totalSeconds / 3600);
    const m = Math.floor((totalSeconds % 3600) / 60);
    const sec = String(totalSeconds % 60).padStart(2, "0");
    return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
}

export function formatBytes(bytes: number): string {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
        value /= 1024;
        unit++;
    }
    return unit === 0 ? `${value} B` : `${value.toFixed(1)} ${units[unit]}`;
}

export function statusInfo(status: DemoStatus): { label: string; hint: string } {
    switch (status) {
        case "complete":
            return { label: "Ready", hint: "Matches the newest replay build found here." };
        case "partial":
            return { label: "Partial", hint: "An unfinished download. It will not play." };
        case "outdated":
            return { label: "Older build", hint: "Recorded on an older game build. It may not play." };
        case "unknown":
            return { label: "Unknown", hint: "The replay header could not be read." };
    }
}

export function totalSize(demos: Demo[]): number {
    return demos.reduce((sum, d) => sum + d.size, 0);
}

export function countByStatus(demos: Demo[]): Record<DemoStatus, number> {
    const counts: Record<DemoStatus, number> = { complete: 0, partial: 0, outdated: 0, unknown: 0 };
    for (const d of demos) counts[d.status]++;
    return counts;
}

export function deleteCopy(p: DeletePreview): { title: string; canRecycle: boolean; notice: string | null } {
    const one = p.count === 1;
    const title = `Delete ${p.count} replay${one ? "" : "s"}?`;
    if (p.recycle === "available") return { title, canRecycle: true, notice: null };
    if (p.recycle === "disabled") {
        return {
            title,
            canRecycle: false,
            notice: "The Recycle Bin is turned off for this drive, so replays can only be deleted permanently. This can't be undone.",
        };
    }
    const room =
        p.binFreeBytes === null
            ? ""
            : ` (${formatBytes(p.totalBytes)} needed, ${formatBytes(p.binFreeBytes)} free there)`;
    return {
        title,
        canRecycle: false,
        notice: `${one ? "This file is" : "These files are"} too large to move to the Recycle Bin${room}. ${
            one ? "It" : "They"
        } can only be deleted permanently, and that can't be undone.`,
    };
}

export function unpinnedNames(demos: Demo[], pinned: Set<number>): string[] {
    return demos.filter((d) => !pinned.has(d.matchId)).map((d) => d.fileName);
}

const DEFAULT_RULES: CleanupRule[] = [
    { id: 1, enabled: false, kind: "olderThanDays", days: 30 },
    { id: 2, enabled: false, kind: "largerThanMb", mb: 1024 },
    { id: 3, enabled: false, kind: "outdated" },
    { id: 4, enabled: false, kind: "partial" },
];

/** The dialog offers one rule per kind; saved values win over the defaults. */
export function withAllRules(saved: CleanupRule[]): CleanupRule[] {
    return DEFAULT_RULES.map((fallback) => saved.find((r) => r.kind === fallback.kind) ?? { ...fallback });
}

export function ruleLabel(rule: CleanupRule): string {
    switch (rule.kind) {
        case "olderThanDays":
            return `Older than ${rule.days} day${rule.days === 1 ? "" : "s"}`;
        case "largerThanMb":
            return rule.mb >= 1024 && rule.mb % 1024 === 0
                ? `Larger than ${rule.mb / 1024} GB`
                : `Larger than ${rule.mb} MB`;
        case "outdated":
            return "Older game build";
        case "partial":
            return "Unfinished downloads";
    }
}

export function gbToMb(gb: number): number {
    return Number.isFinite(gb) ? Math.max(1, Math.round(gb * 1024)) : 1;
}

export function mbToGb(mb: number): number {
    return mb / 1024;
}

export function matchesTotal(matches: { size: number }[]): { count: number; bytes: number } {
    return { count: matches.length, bytes: matches.reduce((sum, m) => sum + m.size, 0) };
}
