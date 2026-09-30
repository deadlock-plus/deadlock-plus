import { describe, expect, it } from "vitest";
import { filterMuted, mutedIds, pageCount, pageSlice, plural, pruneSelection, toggleIds } from "./list";

const A = "76561198035543931";
const B = "76561198000000001";
const C = "76561198000000002";

const dt = (ids: string[]) =>
    `{\n\tusers = \n\t[\n${ids.map((id) => `\t\t{\n\t\t\tflags = 3\n\t\t\tsteamid = ${id}\n\t\t},`).join("\n")}\n\t]\n}`;

describe("mutedIds", () => {
    it("lists newest first", () => {
        expect(mutedIds({ exists: true, text: dt([A, B, C]) })).toEqual([C, B, A]);
    });

    it("is empty when the file is missing or unparsable", () => {
        expect(mutedIds(null)).toEqual([]);
        expect(mutedIds({ exists: false, text: dt([A]) })).toEqual([]);
        expect(mutedIds({ exists: true, text: "garbage" })).toEqual([]);
    });
});

describe("filterMuted", () => {
    const profiles = { [A]: { steamid64: A, name: "Alice", avatar: null, profileUrl: null } };

    it("returns everything for a blank filter", () => {
        expect(filterMuted([A, B], "  ", profiles)).toEqual([A, B]);
    });

    it("matches id, account id and name case-insensitively", () => {
        expect(filterMuted([A, B], "ALICE", profiles)).toEqual([A]);
        expect(filterMuted([A, B], "000001", profiles)).toEqual([B]);
    });

    it("matches the 32-bit account id", () => {
        expect(filterMuted([B], "39734273", {})).toEqual([B]);
    });
});

describe("paging", () => {
    it("has at least one page", () => {
        expect(pageCount(0)).toBe(1);
        expect(pageCount(25)).toBe(1);
        expect(pageCount(26)).toBe(2);
    });

    it("slices by page", () => {
        const ids = Array.from({ length: 30 }, (_, i) => String(i));
        expect(pageSlice(ids, 0)).toHaveLength(25);
        expect(pageSlice(ids, 1)).toEqual(ids.slice(25));
    });
});

describe("selection", () => {
    it("adds and removes ids without mutating", () => {
        const start = new Set([A]);
        const added = toggleIds(start, [B, C], true);
        expect([...added]).toEqual([A, B, C]);
        expect([...start]).toEqual([A]);
        expect([...toggleIds(added, [A, B], false)]).toEqual([C]);
    });

    it("prunes ids that are no longer muted", () => {
        expect([...pruneSelection(new Set([A, B]), [B, C])]).toEqual([B]);
    });
});

describe("plural", () => {
    it("adds an s except for one", () => {
        expect(plural(1, "player")).toBe("1 player");
        expect(plural(0, "player")).toBe("0 players");
        expect(plural(2, "id")).toBe("2 ids");
    });
});
