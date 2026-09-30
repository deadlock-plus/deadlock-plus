import { describe, expect, it } from "vitest";
import {
    allSiblingsBlocked,
    chunk,
    groupPing,
    indexById,
    matchesSearch,
    membersOf,
    siblingNames,
    siblingsToBlock,
} from "./regions";
import type { ServerGroup } from "./types";

function group(id: string, description = id, extra: Partial<ServerGroup> = {}): ServerGroup {
    return {
        id,
        description,
        relayIps: [],
        isCluster: false,
        memberIds: [],
        routingNote: null,
        ...extra,
    } as ServerGroup;
}

describe("membersOf", () => {
    it("resolves member ids, drops unknown ones and sorts by description", () => {
        const byId = indexById([group("b", "Beta"), group("a", "Alpha")]);
        const cluster = group("c", "C", { isCluster: true, memberIds: ["b", "missing", "a"] });
        expect(membersOf(cluster, byId).map((g) => g.id)).toEqual(["a", "b"]);
    });

    it("is empty without member ids", () => {
        expect(membersOf(group("c"), new Map())).toEqual([]);
    });
});

describe("matchesSearch", () => {
    it("matches case-insensitively and matches everything when empty", () => {
        expect(matchesSearch(group("x", "Frankfurt"), "FRANK")).toBe(true);
        expect(matchesSearch(group("x", "Frankfurt"), "paris")).toBe(false);
        expect(matchesSearch(group("x", "Frankfurt"), "")).toBe(true);
    });
});

describe("groupPing", () => {
    const cluster = group("c", "C", { isCluster: true });
    const members = [group("a"), group("b")];

    it("reads a plain group straight from the results", () => {
        expect(groupPing(group("a"), [], { a: 40 })).toBe(40);
        expect(groupPing(group("a"), [], {})).toBeUndefined();
    });

    it("takes the best member ping for a cluster", () => {
        expect(groupPing(cluster, members, { a: 90, b: 30 })).toBe(30);
    });

    it("is null when every member failed and undefined while pending", () => {
        expect(groupPing(cluster, members, { a: null, b: null })).toBeNull();
        expect(groupPing(cluster, members, { a: null })).toBeUndefined();
        expect(groupPing(cluster, [], {})).toBeUndefined();
    });
});

describe("chunk", () => {
    it("splits into batches of the given size", () => {
        expect(chunk([1, 2, 3, 4, 5], 2)).toEqual([[1, 2], [3, 4], [5]]);
        expect(chunk([], 3)).toEqual([]);
    });
});

describe("siblings", () => {
    const note = { relatedGroupIds: ["b", "c"] } as ServerGroup["routingNote"];
    const a = group("a", "A", { routingNote: note });
    const regions = [a, group("b", "B"), group("c", "C")];

    it("lists related regions that are neither blocked nor externally blocked", () => {
        expect(siblingNames(a, regions, new Set(["b"]), new Set())).toEqual(["C"]);
        expect(siblingNames(a, regions, new Set(), new Set(["c"]))).toEqual(["B"]);
        expect(siblingNames(group("z"), regions, new Set(), new Set())).toEqual([]);
    });

    it("reports whether all related regions are covered", () => {
        expect(allSiblingsBlocked(group("z"), new Set(), new Set())).toBe(true);
        expect(allSiblingsBlocked(a, new Set(["b"]), new Set(["c"]))).toBe(true);
        expect(allSiblingsBlocked(a, new Set(["b"]), new Set())).toBe(false);
    });

    it("picks related regions that are not yet blocked", () => {
        expect(siblingsToBlock(a, regions, new Set(["b"])).map((g) => g.id)).toEqual(["c"]);
        expect(siblingsToBlock(group("z"), regions, new Set())).toEqual([]);
    });
});
