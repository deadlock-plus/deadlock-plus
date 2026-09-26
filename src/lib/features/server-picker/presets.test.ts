import { describe, expect, it } from "vitest";
import { diffBlocks, resolveBlockedIds, type Preset } from "./presets";

const ALL = ["fra", "ams", "sgp", "iad"];

const preset = (mode: Preset["mode"], regionIds: string[]): Preset => ({ id: "p", name: "p", mode, regionIds });

describe("resolveBlockedIds", () => {
    it("allow mode blocks everything except the listed regions", () => {
        expect(resolveBlockedIds(preset("allow", ["fra", "ams"]), ALL)).toEqual(["sgp", "iad"]);
    });

    it("block mode blocks only the listed regions", () => {
        expect(resolveBlockedIds(preset("block", ["sgp"]), ALL)).toEqual(["sgp"]);
    });

    it("ignores ids that no longer exist in the relay list", () => {
        expect(resolveBlockedIds(preset("block", ["sgp", "gone"]), ALL)).toEqual(["sgp"]);
        expect(resolveBlockedIds(preset("allow", ["gone"]), ALL)).toEqual(ALL);
    });
});

describe("diffBlocks", () => {
    it("blocks what is missing and unblocks what should be open", () => {
        const { toBlock, toUnblock } = diffBlocks(["sgp", "iad"], new Set(["iad", "fra"]));
        expect(toBlock).toEqual(["sgp"]);
        expect(toUnblock).toEqual(["fra"]);
    });

    it("unblocks relay-level blocks that are not part of the target", () => {
        const { toUnblock } = diffBlocks([], new Set(["some-pop"]));
        expect(toUnblock).toEqual(["some-pop"]);
    });
});
