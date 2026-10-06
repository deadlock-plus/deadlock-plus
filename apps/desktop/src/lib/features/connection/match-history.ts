import { kvGet, kvSet } from "$lib/core/kv";
import type { LivePhase } from "$lib/generated/types/LivePhase";

import type { PingSummary } from "./types";

export const MAX_MATCHES = 50;
/** A phase that flickers out of and back into a match must not leave a record. */
export const MIN_MATCH_MS = 60_000;

export interface MatchRecord {
    id: string;
    startedAt: number;
    endedAt: number;
    /** Direct ping to the relay in ms, one decimal. Loss is not recorded per sample. */
    avg: number;
    worst: number;
    samples: number;
    server: string | null;
}

export interface OpenMatch {
    startedAt: number;
    server?: string | null;
}

/** A match is the span in `inMatch`. Pregame, loading and post-match are outside it. */
export function stepMatch(
    open: OpenMatch | null,
    phase: LivePhase | undefined,
    now: number,
): { open: OpenMatch | null; closed: OpenMatch | null } {
    if (phase === undefined || phase === "unsupported") return { open, closed: null };
    if (phase === "inMatch") return { open: open ?? { startedAt: now }, closed: null };
    return { open: null, closed: open };
}

const round1 = (v: number) => Math.round(v * 10) / 10;

export function finishedMatch(open: OpenMatch, endedAt: number, summary: PingSummary | null): MatchRecord | null {
    if (summary === null || endedAt - open.startedAt < MIN_MATCH_MS) return null;
    return {
        id: String(open.startedAt),
        startedAt: open.startedAt,
        endedAt,
        avg: round1(summary.avg),
        worst: round1(summary.worst),
        samples: summary.samples,
        server: open.server ?? null,
    };
}

export function addMatch(list: MatchRecord[], record: MatchRecord): MatchRecord[] {
    return [record, ...list.filter((m) => m.id !== record.id)].slice(0, MAX_MATCHES);
}

const isNum = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);

function isMatch(v: unknown): v is MatchRecord {
    if (typeof v !== "object" || v === null) return false;
    const r = v as Record<string, unknown>;
    return (
        typeof r.id === "string" &&
        isNum(r.startedAt) &&
        isNum(r.endedAt) &&
        isNum(r.avg) &&
        isNum(r.worst) &&
        isNum(r.samples) &&
        (r.server === null || typeof r.server === "string")
    );
}

export function parseMatches(raw: unknown): MatchRecord[] {
    return Array.isArray(raw) ? raw.filter(isMatch) : [];
}

const STORE = "connection-settings";
const KEY = "matchHistory";

export async function readMatches(): Promise<MatchRecord[]> {
    try {
        return parseMatches(await kvGet<unknown>(STORE, KEY));
    } catch {
        return [];
    }
}

export async function writeMatches(list: MatchRecord[]): Promise<void> {
    await kvSet(STORE, KEY, list);
}
