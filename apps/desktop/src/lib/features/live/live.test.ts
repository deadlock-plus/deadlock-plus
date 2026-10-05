import { describe, expect, it } from "vitest";
import { stateLine } from "./live";
import type { LivePhase } from "$lib/generated/types/LivePhase";

describe("stateLine", () => {
    it("shows nothing when unsupported", () => {
        expect(stateLine("unsupported")).toBeNull();
    });

    it("gives every other phase a distinct line", () => {
        const phases: LivePhase[] = ["gameClosed", "menus", "queuing", "pregame", "inMatch", "postMatch"];
        const lines = phases.map((p) => stateLine(p));
        expect(lines.every((l) => l !== null)).toBe(true);
        expect(new Set(lines.map((l) => l?.key)).size).toBe(phases.length);
    });
});
