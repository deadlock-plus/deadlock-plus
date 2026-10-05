import { describe, expect, it } from "vitest";
import { stateLine } from "./live";
import type { LivePhase } from "$lib/generated/types/LivePhase";

describe("stateLine", () => {
    it("shows nothing when unsupported", () => {
        expect(stateLine("unsupported")).toBeNull();
    });

    it("links reading-off to the memory reading setting", () => {
        expect(stateLine("readingOff")).toEqual({ key: "live.state.reading_off", settingsLink: true });
    });

    it("gives every other phase a distinct plain line without a link", () => {
        const phases: LivePhase[] = ["gameClosed", "menus", "queuing", "pregame", "inMatch", "postMatch"];
        const lines = phases.map((p) => stateLine(p));
        expect(lines.map((l) => l?.settingsLink)).toEqual(phases.map(() => false));
        expect(new Set(lines.map((l) => l?.key)).size).toBe(phases.length);
    });
});
