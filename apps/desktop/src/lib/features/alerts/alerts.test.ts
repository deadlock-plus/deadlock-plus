import { describe, expect, it } from "vitest";
import { formatPublished, kindTone, safeExternalUrl, searchResultKeys, unreadCount, type Alert } from "./alerts";

const alert = (over: Partial<Alert> = {}): Alert => ({
    id: "a",
    title: "t",
    link: "https://forums.playdeadlock.com/threads/x",
    source: "forum",
    published: "2026-09-16T22:41:46Z",
    kind: "Minor Update",
    summary: "",
    image: null,
    read: false,
    ...over,
});

describe("unreadCount", () => {
    it("counts only unread alerts", () => {
        expect(unreadCount([alert(), alert({ read: true }), alert()])).toBe(2);
        expect(unreadCount([])).toBe(0);
    });
});

describe("kindTone", () => {
    it("marks generic patch notes and minor updates as routine", () => {
        expect(kindTone("Patch notes")).toBe("routine");
        expect(kindTone("Minor Update")).toBe("routine");
        expect(kindTone("minor update")).toBe("routine");
    });

    it("marks any other update type as notable", () => {
        expect(kindTone("Matchmaking Update")).toBe("notable");
        expect(kindTone("Major Update")).toBe("notable");
        expect(kindTone("")).toBe("routine");
    });
});

describe("formatPublished", () => {
    it("formats a valid ISO date", () => {
        expect(formatPublished("2026-09-16T22:41:46Z")).toMatch(/2026/);
    });

    it("returns an empty string for a missing or invalid date", () => {
        expect(formatPublished("")).toBe("");
        expect(formatPublished("not a date")).toBe("");
    });
});

describe("safeExternalUrl", () => {
    it("accepts http and https only", () => {
        expect(safeExternalUrl("https://a.test/x")).toBe("https://a.test/x");
        expect(safeExternalUrl("http://a.test/x")).toBe("http://a.test/x");
        expect(safeExternalUrl("file:///C:/Windows/System32/calc.exe")).toBeNull();
        expect(safeExternalUrl("javascript:alert(1)")).toBeNull();
        expect(safeExternalUrl("")).toBeNull();
        expect(safeExternalUrl("nope")).toBeNull();
    });
});

describe("searchResultKeys", () => {
    const hit = (patchId: string, snippet: string) => ({ patchId, snippet });

    it("gives every result a distinct key even when a patch repeats a snippet", () => {
        const keys = searchResultKeys([hit("p1", "-"), hit("p1", "-"), hit("p1", "-"), hit("p2", "-")]);
        expect(new Set(keys).size).toBe(4);
    });

    it("keeps a result's key when unrelated results change around it", () => {
        const before = searchResultKeys([hit("p1", "a"), hit("p2", "b")]);
        const after = searchResultKeys([hit("p3", "c"), hit("p2", "b")]);
        expect(after[1]).toBe(before[1]);
    });

    it("does not let a patch id and snippet run together into another pair", () => {
        const keys = searchResultKeys([hit("ab", "c"), hit("a", "bc")]);
        expect(keys[0]).not.toBe(keys[1]);
    });
});
