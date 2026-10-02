import { describe, expect, it } from "vitest";
import { CATEGORIES, ITEMS, itemsFor, matchingItems, matchingCategories } from "./catalog";

describe("catalog", () => {
    it("assigns every item to a known category", () => {
        const ids = new Set(CATEGORIES.map((c) => c.id));
        for (const item of ITEMS) expect(ids.has(item.category)).toBe(true);
    });

    it("titles the autostart item for the given platform", () => {
        const title = (p: "windows" | "macos" | "linux") => itemsFor(p).find((i) => i.id === "autostart")?.title;
        expect(title("windows")).toBe("Start with Windows");
        expect(title("macos")).toBe("Start with macOS");
        expect(title("linux")).toBe("Start at login");
    });

    it("lists instant match results only on Windows", () => {
        const has = (p: "windows" | "macos" | "linux") => itemsFor(p).some((i) => i.id === "postgame-capture");
        expect(has("windows")).toBe(true);
        expect(has("macos")).toBe(false);
        expect(has("linux")).toBe(false);
    });

    it("has unique item ids", () => {
        expect(new Set(ITEMS.map((i) => i.id)).size).toBe(ITEMS.length);
    });

    it("matches nothing-filtered for an empty query", () => {
        expect(matchingItems("")).toBeNull();
        expect(matchingItems("   ")).toBeNull();
    });

    it("matches on the title, case-insensitively", () => {
        expect(matchingItems("ACCESSIBLE")?.has("accessible-font")).toBe(true);
    });

    it("matches on keywords that are not in the title", () => {
        expect(matchingItems("dyslexia")?.has("accessible-font")).toBe(true);
        expect(matchingItems("sign in")?.has("autostart")).toBe(true);
    });

    it("finds the background work setting by the tasks it covers", () => {
        expect(matchingItems("scan addons at launch")?.has("background-jobs")).toBe(true);
        expect(matchingItems("index patch notes automatically")?.has("background-jobs")).toBe(true);
        expect(matchingItems("pause game running")?.has("background-jobs")).toBe(true);
    });

    it("no longer lists the launch scan and indexing switches on their own", () => {
        const ids = ITEMS.map((i) => i.id);
        expect(ids).not.toContain("auto-scan-addons");
        expect(ids).not.toContain("auto-index-patch-notes");
    });

    it("requires every word to match", () => {
        expect(matchingItems("tray quit")?.has("close-to-tray")).toBe(true);
        expect(matchingItems("tray zzzz")?.size).toBe(0);
    });

    it("lists categories that have a match, in catalog order", () => {
        const ids = matchingCategories(matchingItems("steam"));
        expect(ids).toContain("notifications");
        expect(ids).toEqual(CATEGORIES.map((c) => c.id).filter((id) => ids.includes(id)));
    });

    it("lists every category when there is no filter", () => {
        expect(matchingCategories(null)).toEqual(CATEGORIES.map((c) => c.id));
    });
});
