import { describe, expect, it } from "vitest";
import {
    calibratedOffset,
    exitLagAvailable,
    exitLagSaved,
    formatOffset,
    gapVariant,
    historySeries,
    mergeTail,
    HISTORY_SHOWN,
    routedAverage,
} from "./connection";
import type { HistoryPoint } from "./types";

const point = (raw: number | null, exit: number | null): HistoryPoint => ({ raw, exit }) as HistoryPoint;

describe("gapVariant", () => {
    it("grades the longest inbound gap", () => {
        expect(gapVariant(99)).toBe("success");
        expect(gapVariant(100)).toBe("warning");
        expect(gapVariant(249)).toBe("warning");
        expect(gapVariant(250)).toBe("destructive");
    });
});

describe("calibratedOffset", () => {
    it("rounds to one decimal", () => {
        expect(calibratedOffset("30", 22.34)).toBe(7.7);
        expect(calibratedOffset("20", 22.34)).toBe(-2.3);
    });
    it("rejects bad input or a missing measurement", () => {
        expect(calibratedOffset("", 20)).toBeNull();
        expect(calibratedOffset("abc", 20)).toBeNull();
        expect(calibratedOffset("30", null)).toBeNull();
        expect(calibratedOffset("30", undefined)).toBeNull();
    });
});

describe("routedAverage and exitLagSaved", () => {
    it("adds the offset to the exit ping", () => {
        expect(routedAverage(20, 5)).toBe(25);
        expect(routedAverage(null, 5)).toBeNull();
    });
    it("is raw minus routed, null when either is missing", () => {
        expect(exitLagSaved(50, 30)).toBe(20);
        expect(exitLagSaved(30, 50)).toBe(-20);
        expect(exitLagSaved(null, 30)).toBeNull();
        expect(exitLagSaved(50, null)).toBeNull();
    });
});

describe("formatOffset", () => {
    it("signs positive and zero values", () => {
        expect(formatOffset(3)).toBe("+3");
        expect(formatOffset(0)).toBe("+0");
        expect(formatOffset(-2.5)).toBe("-2.5");
    });
});

describe("historySeries", () => {
    it("keeps only the newest points and applies the offset to the exit line", () => {
        const history = Array.from({ length: HISTORY_SHOWN + 5 }, (_, i) => point(i, i % 2 ? null : i));
        const { shown, series } = historySeries(history, 10);
        expect(shown).toHaveLength(HISTORY_SHOWN);
        expect(series[0].values[0]).toBe(5);
        expect(series[1].values[0]).toBe(null);
        expect(series[1].values[1]).toBe(16);
    });

    it("draws only the direct route when ExitLag is not offered", () => {
        const { series } = historySeries([point(20, 30)], 0, false);
        expect(series.map((s) => s.label)).toEqual(["Ping"]);
        expect(series[0].values).toEqual([20]);
    });
});

describe("ExitLag availability", () => {
    it("is a Windows-only app", () => {
        expect(exitLagAvailable("windows")).toBe(true);
        expect(exitLagAvailable("macos")).toBe(false);
        expect(exitLagAvailable("linux")).toBe(false);
    });
});

describe("mergeTail", () => {
    const at = (t: number): HistoryPoint => ({ t, raw: t, exit: null });

    it("appends the new points", () => {
        expect(mergeTail([at(1), at(2)], [at(3)]).map((p) => p.t)).toEqual([1, 2, 3]);
    });

    it("keeps the same array when nothing is new", () => {
        const history = [at(1)];
        expect(mergeTail(history, [])).toBe(history);
    });

    it("drops points it already has", () => {
        expect(mergeTail([at(1), at(2)], [at(2), at(3)]).map((p) => p.t)).toEqual([1, 2, 3]);
    });

    it("keeps only the newest HISTORY_SHOWN points", () => {
        const history = Array.from({ length: HISTORY_SHOWN }, (_, i) => at(i + 1));
        const next = mergeTail(history, [at(HISTORY_SHOWN + 1), at(HISTORY_SHOWN + 2)]);
        expect(next).toHaveLength(HISTORY_SHOWN);
        expect(next[0].t).toBe(3);
        expect(next.at(-1)?.t).toBe(HISTORY_SHOWN + 2);
    });
});
