import { networkHistoryPoints, networkHistoryRange } from "$lib/features/connection/api";
import type { HistoryPoint, PingSummary } from "$lib/features/connection/types";
import type { LinePoint } from "./deep-dive/chart";
import type { MatchDetail } from "./detail";

export const MAX_PING_POINTS = 300;

export interface PingSource {
    summary(startMs: number, endMs: number): Promise<PingSummary | null>;
    points(startMs: number, endMs: number, maxPoints: number): Promise<HistoryPoint[]>;
}

export interface MatchPing {
    summary: PingSummary;
    /** Runs of consecutive direct pings, in seconds from the match start. A dropped sample ends a run. */
    segments: LinePoint[][];
    durationS: number;
}

export const livePingSource: PingSource = { summary: networkHistoryRange, points: networkHistoryPoints };

export function matchWindowMs(detail: Pick<MatchDetail, "startTime" | "durationS">) {
    return { startMs: detail.startTime * 1000, endMs: (detail.startTime + detail.durationS) * 1000 };
}

export function pingSegments(points: readonly HistoryPoint[], startMs: number): LinePoint[][] {
    const segments: LinePoint[][] = [];
    let run: LinePoint[] | null = null;
    for (const p of points) {
        if (p.raw === null) {
            run = null;
            continue;
        }
        if (!run) {
            run = [];
            segments.push(run);
        }
        run.push({ t: (p.t - startMs) / 1000, v: p.raw });
    }
    return segments;
}

/** `null` when the log holds no direct ping inside the match window. */
export async function loadMatchPing(
    detail: Pick<MatchDetail, "startTime" | "durationS">,
    source: PingSource = livePingSource,
): Promise<MatchPing | null> {
    const { startMs, endMs } = matchWindowMs(detail);
    const [summary, points] = await Promise.all([
        source.summary(startMs, endMs),
        source.points(startMs, endMs, MAX_PING_POINTS),
    ]);
    if (!summary) return null;
    return { summary, segments: pingSegments(points, startMs), durationS: detail.durationS };
}
