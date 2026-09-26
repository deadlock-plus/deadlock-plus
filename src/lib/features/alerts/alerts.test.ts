import { describe, expect, it } from "vitest";
import { formatPublished, kindTone, safeExternalUrl, unreadCount, type Alert } from "./alerts";

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
