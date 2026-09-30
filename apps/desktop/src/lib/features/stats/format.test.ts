import { describe, expect, it } from "vitest";
import { etaText, pct, signed } from "./format";

describe("signed", () => {
    it("prefixes positives only", () => {
        expect(signed(5)).toBe("+5");
        expect(signed(0)).toBe("0");
        expect(signed(-3)).toBe("-3");
    });
});

describe("pct", () => {
    it("rounds a fraction to a percent", () => {
        expect(pct(0.456)).toBe("46%");
        expect(pct(0)).toBe("0%");
    });
    it("shows a dash for no data", () => {
        expect(pct(null)).toBe("-");
    });
});

describe("etaText", () => {
    it("handles no climb", () => {
        expect(etaText({ points: 1000, daysLow: null, daysHigh: null })).toBe("Not climbing at your recent pace");
    });
    it("handles an open upper bound", () => {
        expect(etaText({ points: 1000, daysLow: 12, daysHigh: null })).toBe("12 days or more");
    });
    it("handles a single day count", () => {
        expect(etaText({ points: 1000, daysLow: 1, daysHigh: 1 })).toBe("About 1 day");
        expect(etaText({ points: 1000, daysLow: 4, daysHigh: 4 })).toBe("About 4 days");
    });
    it("handles a range", () => {
        expect(etaText({ points: 1000, daysLow: 3, daysHigh: 6 })).toBe("3 to 6 days");
    });
});
