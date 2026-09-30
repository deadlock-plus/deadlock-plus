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
        label: trimmed || `Run ${new Date(now).toISOString().slice(0, 10)}`,
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
    label: string;
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
        label: string,
        unit: string,
        av: number,
        bv: number,
        higherIsBetter: boolean,
        tie = relativeTie,
    ): CompareRow => ({ key, label, unit, a: av, b: bv, higherIsBetter, better: judge(av, bv, higherIsBetter, tie) });

    const [x, y] = [a.stats, b.stats];
    return [
        row("avgFps", "Average FPS", "fps", 1000 / x.avgMs, 1000 / y.avgMs, true),
        row("low1pctFps", "1% low FPS", "fps", x.low1pctFps, y.low1pctFps, true),
        row("low01pctFps", "0.1% low FPS", "fps", x.low01pctFps, y.low01pctFps, true),
        row("medianMs", "Median frametime", "ms", x.medianMs, y.medianMs, false),
        row("p95Ms", "95th percentile", "ms", x.p95Ms, y.p95Ms, false),
        row("p99Ms", "99th percentile", "ms", x.p99Ms, y.p99Ms, false),
        row(
            "spikes",
            "Spikes per minute",
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

export function formatValue(v: number, unit: string): string {
    return unit === "ms" ? v.toFixed(2) : unit === "/min" ? v.toFixed(1) : v.toFixed(0);
}

export function comparisonReport(a: SavedRun, b: SavedRun): string {
    const rows = compareRuns(a, b);
    const width = Math.max(...rows.map((r) => r.label.length));
    const table = rows.map((r) => {
        const better = r.better === "tie" ? "tie" : r.better.toUpperCase();
        return `${r.label.padEnd(width)}  ${formatValue(r.a, r.unit).padStart(8)}  ${formatValue(r.b, r.unit).padStart(8)}  ${better}`;
    });
    const { onlyInA, onlyInB } = addonDiff(a, b);
    const addons: string[] = [];
    if (onlyInA.length > 0) addons.push(`Only on in ${a.label}: ${onlyInA.join(", ")}`);
    if (onlyInB.length > 0) addons.push(`Only on in ${b.label}: ${onlyInB.join(", ")}`);
    if (addons.length === 0)
        addons.push("Both runs had the same addons on, so any difference comes from something else.");
    const minutes = (r: SavedRun) => `${(r.stats.durationMs / 60_000).toFixed(1)} min, ${r.stats.frameCount} frames`;
    return [
        "Deadlock+ frametime comparison",
        "",
        `Run A: ${a.label} (${minutes(a)})`,
        `Run B: ${b.label} (${minutes(b)})`,
        "",
        `${"".padEnd(width)}  ${"A".padStart(8)}  ${"B".padStart(8)}  Better`,
        ...table,
        "",
        ...addons,
        "",
        "Differences under 3% count as a tie. Frame pacing varies between matches, so a difference is consistent with a mod being the cause, not proof of it.",
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
