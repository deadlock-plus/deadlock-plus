import { describe, expect, it } from "vitest";
import { clearCopy, ENTRY_IDS, ENTRY_META, knownTotal, type EntryId } from "./storage";

describe("entry metadata", () => {
    it("has a label and description for every entry id", () => {
        for (const id of ENTRY_IDS) {
            expect(ENTRY_META[id].label.length).toBeGreaterThan(0);
            expect(ENTRY_META[id].description.length).toBeGreaterThan(0);
        }
    });

    it("links only the replays entry to another page", () => {
        const linked = ENTRY_IDS.filter((id) => ENTRY_META[id].link);
        expect(linked).toEqual(["replays"]);
        expect(ENTRY_META.replays.link).toBe("/demos");
    });
});

describe("knownTotal", () => {
    it("adds the sizes that have arrived and skips the ones still loading", () => {
        const sizes: Partial<Record<EntryId, number>> = { replays: 100, "shader-cache": 20 };
        expect(knownTotal(sizes)).toBe(120);
    });

    it("is zero when nothing has loaded", () => {
        expect(knownTotal({})).toBe(0);
    });
});

describe("clearCopy", () => {
    it("names the entry and what it frees", () => {
        const copy = clearCopy("shader-cache", 5 * 1024 * 1024);
        expect(copy.title).toContain("shader cache");
        expect(copy.body).toContain("5.0 MB");
    });

    it("says the file cannot be restored", () => {
        for (const id of ["shader-cache", "console-log", "voice-ban-backups"] as const) {
            expect(clearCopy(id, 1).body).toMatch(/can't be undone/i);
        }
    });
});
