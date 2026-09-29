import { describe, expect, it } from "vitest";
import { bestPing } from "./estimate";

describe("bestPing", () => {
    it("takes the lowest reply and skips missing ones", () => {
        expect(bestPing([120, undefined, 80, null])).toBe(80);
    });

    it("is null when nothing replied", () => {
        expect(bestPing([null, undefined])).toBeNull();
    });
});
