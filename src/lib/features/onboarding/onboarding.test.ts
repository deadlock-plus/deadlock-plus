import { describe, expect, it } from "vitest";
import { gameStatus, needsOnboarding } from "./onboarding";

describe("needsOnboarding", () => {
    it("runs when nothing was stored", () => {
        expect(needsOnboarding(undefined)).toBe(true);
        expect(needsOnboarding(null)).toBe(true);
    });

    it("runs again for a stored false", () => {
        expect(needsOnboarding(false)).toBe(true);
    });

    it("stays away once finished", () => {
        expect(needsOnboarding(true)).toBe(false);
    });
});

describe("gameStatus", () => {
    it("reports the install folder when found", () => {
        expect(gameStatus("C:\Steam\Deadlock")).toEqual({ found: true, path: "C:\Steam\Deadlock" });
    });

    it("reports a miss for null, undefined and empty", () => {
        for (const dir of [null, undefined, ""]) expect(gameStatus(dir)).toEqual({ found: false, path: null });
    });
});
