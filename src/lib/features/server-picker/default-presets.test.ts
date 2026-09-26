import { describe, expect, it } from "vitest";
import { DEFAULT_PRESETS } from "./default-presets";
import { resolveBlockedIds } from "./presets";

const byName = (name: string) => DEFAULT_PRESETS.find((p) => p.name === name)!;

describe("DEFAULT_PRESETS", () => {
    it("ships the six built-in presets", () => {
        expect(DEFAULT_PRESETS.map((p) => p.name)).toEqual(["Asia", "EU All", "EU West", "USA", "USA East", "ZA"]);
    });

    it("has unique ids and no empty or repeated region lists", () => {
        expect(new Set(DEFAULT_PRESETS.map((p) => p.id)).size).toBe(DEFAULT_PRESETS.length);
        for (const p of DEFAULT_PRESETS) {
            expect(p.regionIds.length, p.name).toBeGreaterThan(0);
            expect(new Set(p.regionIds).size, p.name).toBe(p.regionIds.length);
        }
    });

    it("keeps only the listed regions open", () => {
        for (const p of DEFAULT_PRESETS) expect(p.mode, p.name).toBe("allow");
        const all = ["frankfurt", "ams", "iad", "jnb", "sgp"];
        expect(resolveBlockedIds(byName("ZA"), all)).toEqual(["frankfurt", "ams", "iad", "sgp"]);
    });

    it("keeps USA East inside USA and EU West inside EU All", () => {
        const within = (inner: string, outer: string) =>
            byName(inner).regionIds.every((id) => byName(outer).regionIds.includes(id));
        expect(within("USA East", "USA")).toBe(true);
        expect(within("EU West", "EU All")).toBe(true);
    });
});
