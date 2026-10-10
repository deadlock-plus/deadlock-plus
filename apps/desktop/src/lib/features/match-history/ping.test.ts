import { describe, expect, it, vi } from "vitest";
import { loadMatchPing, MAX_PING_POINTS, matchWindowMs, pingSegments } from "./ping";

describe("matchWindowMs", () => {
    it("turns start time and duration in seconds into a millisecond window", () => {
        expect(matchWindowMs({ startTime: 1000, durationS: 90 })).toEqual({ startMs: 1_000_000, endMs: 1_090_000 });
    });
});

describe("pingSegments", () => {
    it("places points in seconds from the match start", () => {
        const segs = pingSegments(
            [
                { t: 10_000, raw: 20 },
                { t: 12_500, raw: 30 },
            ],
            10_000,
        );
        expect(segs).toEqual([
            [
                { t: 0, v: 20 },
                { t: 2.5, v: 30 },
            ],
        ]);
    });

    it("splits the line at dropped samples instead of drawing zero", () => {
        const segs = pingSegments(
            [
                { t: 0, raw: 20 },
                { t: 1000, raw: null },
                { t: 2000, raw: 40 },
                { t: 3000, raw: 50 },
            ],
            0,
        );
        expect(segs).toEqual([
            [{ t: 0, v: 20 }],
            [
                { t: 2, v: 40 },
                { t: 3, v: 50 },
            ],
        ]);
    });

    it("is empty without points", () => {
        expect(pingSegments([], 0)).toEqual([]);
    });
});

describe("loadMatchPing", () => {
    const detail = { startTime: 100, durationS: 60 };

    it("asks for the window once for the summary and once for a capped point set", async () => {
        const summary = vi.fn().mockResolvedValue({ avg: 30, worst: 90, samples: 55 });
        const points = vi.fn().mockResolvedValue([{ t: 100_000, raw: 30 }]);
        const got = await loadMatchPing(detail, { summary, points });
        expect(summary).toHaveBeenCalledWith(100_000, 160_000);
        expect(points).toHaveBeenCalledWith(100_000, 160_000, MAX_PING_POINTS);
        expect(got?.summary).toEqual({ avg: 30, worst: 90, samples: 55 });
        expect(got?.segments).toHaveLength(1);
    });

    it("is null when no ping falls inside the window", async () => {
        const got = await loadMatchPing(detail, {
            summary: async () => null,
            points: async () => [{ t: 100_000, raw: null }],
        });
        expect(got).toBeNull();
    });
});
