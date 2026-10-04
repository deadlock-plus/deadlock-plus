import { formatNumber, t } from "$lib/core/i18n.svelte";
import { kvGet, kvSet } from "$lib/core/kv";

import type { FrameStats } from "./api";

export const MAX_RUNS = 50;

const TIE_RATIO = 0.03;
const SPIKE_TIE_PER_MIN = 0.5;

export interface SavedRun {
    id: string;
    label: string;
    savedAt: number;
    /** Enabled addons when the run was saved, sorted. */
    addons: string[];
    stats: FrameStats;
}

export function makeRun(label: string, addons: string[], stats: FrameStats, now: number, id: string): SavedRun {
    const trimmed = label.trim();
    return {
        id,
        label: trimmed || t("performance.runs.default_label", { date: new Date(now).toISOString().slice(0, 10) }),
        savedAt: now,
        addons: [...addons].sort(),
        stats,
    };
}

export function addRun(runs: SavedRun[], run: SavedRun): SavedRun[] {
    return [run, ...runs].slice(0, MAX_RUNS);
}

export function removeRun(runs: SavedRun[], id: string): SavedRun[] {
    return runs.filter((r) => r.id !== id);
}

const isNum = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);

function isStats(v: unknown): v is FrameStats {
    if (typeof v !== "object" || v === null) return false;
    const s = v as Record<string, unknown>;
    const numeric = [
        "frameCount",
        "durationMs",
        "backgroundMs",
        "avgMs",
        "medianMs",
        "p95Ms",
        "p99Ms",
        "p999Ms",
        "maxMs",
        "low1pctFps",
        "low01pctFps",
    ];
    return (
        numeric.every((k) => isNum(s[k])) &&
        Array.isArray(s.spikes) &&
        s.spikes.every(
            (x) => typeof x === "object" && x !== null && isNum((x as any).atMs) && isNum((x as any).frametimeMs),
        )
    );
}

function isRun(v: unknown): v is SavedRun {
    if (typeof v !== "object" || v === null) return false;
    const r = v as Record<string, unknown>;
    return (
        typeof r.id === "string" &&
        typeof r.label === "string" &&
        isNum(r.savedAt) &&
        Array.isArray(r.addons) &&
        r.addons.every((a) => typeof a === "string") &&
        isStats(r.stats)
    );
}

export function parseRuns(raw: unknown): SavedRun[] {
    return Array.isArray(raw) ? raw.filter(isRun) : [];
}

export type Better = "a" | "b" | "tie";

export interface CompareRow {
    key: "avgFps" | "low1pctFps" | "low01pctFps" | "medianMs" | "p95Ms" | "p99Ms" | "spikes";
    unit: string;
    a: number;
    b: number;
    higherIsBetter: boolean;
    better: Better;
}

function judge(a: number, b: number, higherIsBetter: boolean, tie: (a: number, b: number) => boolean): Better {
    if (tie(a, b) || a === b) return "tie";
    return a > b === higherIsBetter ? "a" : "b";
}

const relativeTie = (a: number, b: number) => Math.abs(a - b) <= Math.max(Math.abs(a), Math.abs(b)) * TIE_RATIO;

export function spikesPerMinute(s: FrameStats): number {
    return s.durationMs > 0 ? (s.spikes.length / s.durationMs) * 60_000 : 0;
}

export function compareRuns(a: SavedRun, b: SavedRun): CompareRow[] {
    const row = (
        key: CompareRow["key"],
        unit: string,
        av: number,
        bv: number,
        higherIsBetter: boolean,
        tie = relativeTie,
    ): CompareRow => ({ key, unit, a: av, b: bv, higherIsBetter, better: judge(av, bv, higherIsBetter, tie) });

    const [x, y] = [a.stats, b.stats];
    return [
        row("avgFps", "fps", 1000 / x.avgMs, 1000 / y.avgMs, true),
        row("low1pctFps", "fps", x.low1pctFps, y.low1pctFps, true),
        row("low01pctFps", "fps", x.low01pctFps, y.low01pctFps, true),
        row("medianMs", "ms", x.medianMs, y.medianMs, false),
        row("p95Ms", "ms", x.p95Ms, y.p95Ms, false),
        row("p99Ms", "ms", x.p99Ms, y.p99Ms, false),
        row(
            "spikes",
            "/min",
            spikesPerMinute(x),
            spikesPerMinute(y),
            false,
            (p, q) => Math.abs(p - q) < SPIKE_TIE_PER_MIN,
        ),
    ];
}

export function addonDiff(a: SavedRun, b: SavedRun): { onlyInA: string[]; onlyInB: string[] } {
    return {
        onlyInA: a.addons.filter((x) => !b.addons.includes(x)),
        onlyInB: b.addons.filter((x) => !a.addons.includes(x)),
    };
}

export function compareLabel(key: CompareRow["key"]): string {
    switch (key) {
        case "avgFps":
            return t("performance.metrics.avg_fps");
        case "low1pctFps":
            return t("performance.metrics.low_1pct_fps");
        case "low01pctFps":
            return t("performance.metrics.low_01pct_fps");
        case "medianMs":
            return t("performance.metrics.median_frametime");
        case "p95Ms":
            return t("performance.metrics.p95");
        case "p99Ms":
            return t("performance.metrics.p99");
        case "spikes":
            return t("performance.metrics.spikes_per_minute");
    }
}

export function formatValue(v: number, unit: string): string {
    const digits = unit === "ms" ? 2 : unit === "/min" ? 1 : 0;
    return formatNumber(v, { minimumFractionDigits: digits, maximumFractionDigits: digits, useGrouping: false });
}

export function comparisonReport(a: SavedRun, b: SavedRun): string {
    const rows = compareRuns(a, b).map((r) => ({ ...r, label: compareLabel(r.key) }));
    const width = Math.max(...rows.map((r) => r.label.length));
    const table = rows.map((r) => {
        const better = r.better === "tie" ? t("performance.report.tie") : r.better.toUpperCase();
        return `${r.label.padEnd(width)}  ${formatValue(r.a, r.unit).padStart(8)}  ${formatValue(r.b, r.unit).padStart(8)}  ${better}`;
    });
    const { onlyInA, onlyInB } = addonDiff(a, b);
    const addons: string[] = [];
    if (onlyInA.length > 0) addons.push(t("performance.only_on_in", { run: a.label, addons: onlyInA.join(", ") }));
    if (onlyInB.length > 0) addons.push(t("performance.only_on_in", { run: b.label, addons: onlyInB.join(", ") }));
    if (addons.length === 0) addons.push(t("performance.same_addons"));
    const minutes = (r: SavedRun) =>
        t("performance.report.run_length", {
            minutes: formatNumber(r.stats.durationMs / 60_000, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
            frames: r.stats.frameCount,
        });
    return [
        t("performance.report.title"),
        "",
        t("performance.report.run_a", { label: a.label, length: minutes(a) }),
        t("performance.report.run_b", { label: b.label, length: minutes(b) }),
        "",
        `${"".padEnd(width)}  ${"A".padStart(8)}  ${"B".padStart(8)}  ${t("performance.report.better")}`,
        ...table,
        "",
        ...addons,
        "",
        t("performance.report.disclaimer"),
        "",
    ].join("\n");
}

const STORE = "frame-runs";
const KEY = "runs";

export async function readRuns(): Promise<SavedRun[]> {
    try {
        return parseRuns(await kvGet<unknown>(STORE, KEY));
    } catch {
        return [];
    }
}

export async function writeRuns(runs: SavedRun[]): Promise<void> {
    await kvSet(STORE, KEY, runs);
}
