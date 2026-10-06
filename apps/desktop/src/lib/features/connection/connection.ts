import { t } from "$lib/core/i18n.svelte";
import type { BadgeVariant } from "$lib/ui/badge.svelte";
import type { Platform } from "$lib/core/platform";
import type { HistoryPoint } from "./types";

export const HISTORY_SHOWN = 300;

export function exitLagAvailable(p: Platform): boolean {
    return p === "windows";
}

export function gapVariant(maxGapMs: number): BadgeVariant {
    if (maxGapMs < 100) return "success";
    if (maxGapMs < 250) return "warning";
    return "destructive";
}

/** Offset between the ping ExitLag shows and our measured exit-server ping, or null when it can't be computed. */
export function calibratedOffset(entered: string, measured: number | null | undefined): number | null {
    const value = Number.parseFloat(entered);
    if (!Number.isFinite(value) || measured == null) return null;
    return Math.round((value - measured) * 10) / 10;
}

export function routedAverage(exitAvg: number | null | undefined, offset: number): number | null {
    return exitAvg != null ? exitAvg + offset : null;
}

export function exitLagSaved(rawAvg: number | null, routedAvg: number | null): number | null {
    return rawAvg != null && routedAvg != null ? rawAvg - routedAvg : null;
}

export function formatOffset(offset: number): string {
    return `${offset >= 0 ? "+" : ""}${offset}`;
}

export function historySeries(history: HistoryPoint[], offset: number, withExitLag = true) {
    const shown = history.slice(-HISTORY_SHOWN);
    const direct = shown.map((p) => p.raw);
    if (!withExitLag)
        return {
            shown,
            series: [{ label: t("connection.series.ping"), color: "var(--muted-foreground)", values: direct }],
        };
    return {
        shown,
        series: [
            { label: t("connection.series.without"), color: "var(--muted-foreground)", values: direct },
            {
                label: t("connection.series.with"),
                color: "var(--success)",
                values: shown.map((p) => (p.exit == null ? null : p.exit + offset)),
            },
        ],
    };
}
