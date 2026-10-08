import { describe, expect, it } from "vitest";
import { seriesBounds, sparkPath } from "./sparkline";

describe("seriesBounds", () => {
    it("pads the range and ignores nulls", () => {
        expect(seriesBounds([[10, null, 30]])).toEqual({ min: 5, max: 35 });
    });

    it("falls back when there is no data", () => {
        expect(seriesBounds([[null], []])).toEqual({ min: 0, max: 1 });
    });

    it("spans every series", () => {
        expect(seriesBounds([[20], [40]])).toEqual({ min: 15, max: 45 });
    });

    it("handles more points than a spread could take", () => {
        const big = Array.from({ length: 300_000 }, (_, i) => i % 50);
        expect(seriesBounds([big])).toEqual({ min: 0, max: 54 });
    });
});

describe("sparkPath", () => {
    const geo = { width: 100, height: 54, pad: 4, min: 0, max: 50 };

    it("draws a line across the width", () => {
        expect(sparkPath([0, 50], geo)).toBe("M4.0 50.0 L96.0 4.0");
    });

    it("lifts the pen over gaps", () => {
        expect(sparkPath([0, null, 50], geo)).toBe("M4.0 50.0 M96.0 4.0");
    });

    it("is empty without values", () => {
        expect(sparkPath([], geo)).toBe("");
    });
});
