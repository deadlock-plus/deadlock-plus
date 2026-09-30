import { describe, expect, it } from "vitest";
import { demoTitle, filterDemos, pageCount, pageSlice, pinnedCount, runPool, type Demo } from "./list";

const demo = (matchId: number, status: Demo["status"] = "complete"): Demo =>
    ({ matchId, status, fileName: `${matchId}.dem`, size: 1, modifiedMs: 0, buildNum: null }) as unknown as Demo;

describe("filterDemos", () => {
    const demos = [demo(1, "complete"), demo(2, "partial"), demo(3, "complete")];
    const pinned = new Set([3]);

    it("returns everything for all", () => {
        expect(filterDemos(demos, "all", pinned)).toHaveLength(3);
    });
    it("filters by status", () => {
        expect(filterDemos(demos, "partial", pinned).map((d) => d.matchId)).toEqual([2]);
    });
    it("filters by pinned", () => {
        expect(filterDemos(demos, "pinned", pinned).map((d) => d.matchId)).toEqual([3]);
    });
});

describe("pinnedCount", () => {
    it("counts only demos present in the list", () => {
        expect(pinnedCount([demo(1), demo(2)], new Set([2, 99]))).toBe(1);
    });
});

describe("paging", () => {
    it("has at least one page", () => {
        expect(pageCount(0, 25)).toBe(1);
    });
    it("rounds up", () => {
        expect(pageCount(26, 25)).toBe(2);
    });
    it("slices a page", () => {
        expect(pageSlice([1, 2, 3, 4, 5], 1, 2)).toEqual([3, 4]);
    });
});

describe("demoTitle", () => {
    it("prefers the hero name", () => {
        expect(demoTitle("Haze", { heroId: 7 }, 5)).toBe("Haze");
    });
    it("falls back to the hero id", () => {
        expect(demoTitle(undefined, { heroId: 7 }, 5)).toBe("Hero 7");
    });
    it("falls back to the match id", () => {
        expect(demoTitle(undefined, null, 5)).toBe("Match 5");
    });
});

describe("runPool", () => {
    it("processes every item within the concurrency limit", async () => {
        let active = 0;
        let peak = 0;
        const seen: number[] = [];
        await runPool([1, 2, 3, 4, 5, 6, 7], 3, async (n) => {
            active++;
            peak = Math.max(peak, active);
            await Promise.resolve();
            seen.push(n);
            active--;
        });
        expect(seen.sort()).toEqual([1, 2, 3, 4, 5, 6, 7]);
        expect(peak).toBeLessThanOrEqual(3);
    });
    it("handles an empty list", async () => {
        await runPool([], 3, async () => {});
    });
});
