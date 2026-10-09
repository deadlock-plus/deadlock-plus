import { describe, expect, it } from "vitest";
import { needsRefresh, parseHeroCache, slimHeroes } from "./heroes";

describe("needsRefresh", () => {
    const hour = 60 * 60 * 1000;
    const known = { 2: { id: 2, name: "Seven", icon: null, portrait: null, artIcon: null, hideoutLine: null } };

    it("refetches when a wanted hero is missing from a fresh cache", () => {
        expect(needsRefresh(known, [2, 88], null, 5000)).toBe(true);
    });

    it("leaves a cache alone when every wanted hero is present", () => {
        expect(needsRefresh(known, [2], null, 5000)).toBe(false);
        expect(needsRefresh(known, [], null, 5000)).toBe(false);
    });

    it("waits an hour after the last attempt before trying again", () => {
        expect(needsRefresh(known, [88], 1000, 1000 + hour - 1)).toBe(false);
        expect(needsRefresh(known, [88], 1000, 1000 + hour)).toBe(true);
    });
});

describe("slimHeroes", () => {
    it("keeps id, name, the small webp icon and the png art for Discord", () => {
        const out = slimHeroes([
            {
                id: 65,
                name: "Venator",
                images: { icon_image_small_webp: "u.webp", icon_hero_card: "card.png", icon_image_small: "sm.png" },
                hideout_rich_presence: "Plotting in the Hideout",
            },
            { id: 2, name: "Seven" },
        ]);
        expect(out[65]).toEqual({
            id: 65,
            name: "Venator",
            icon: "u.webp",
            portrait: "card.png",
            artIcon: "sm.png",
            hideoutLine: "Plotting in the Hideout",
        });
        expect(out[2]).toEqual({
            id: 2,
            name: "Seven",
            icon: null,
            portrait: null,
            artIcon: null,
            hideoutLine: null,
        });
    });
});

describe("parseHeroCache", () => {
    const raw = JSON.stringify({
        at: 1000,
        heroes: { 2: { id: 2, name: "Seven", icon: null, portrait: null, artIcon: null, hideoutLine: null } },
    });
    const week = 7 * 24 * 60 * 60 * 1000;

    it("returns a fresh cache", () => {
        expect(parseHeroCache(raw, 1000 + week - 1, false)?.[2].name).toBe("Seven");
    });

    it("drops an expired cache unless stale data is acceptable", () => {
        expect(parseHeroCache(raw, 1000 + week, false)).toBeNull();
        expect(parseHeroCache(raw, 1000 + week, true)?.[2].name).toBe("Seven");
    });

    it("treats a cache saved before the hideout line existed as expired", () => {
        const old = JSON.stringify({
            at: 1000,
            heroes: { 2: { id: 2, name: "Seven", icon: null, portrait: null, artIcon: null } },
        });
        expect(parseHeroCache(old, 1001, false)).toBeNull();
        expect(parseHeroCache(old, 1001, true)?.[2].name).toBe("Seven");
    });

    it("treats a cache saved before the art fields existed as expired", () => {
        const old = JSON.stringify({ at: 1000, heroes: { 2: { id: 2, name: "Seven", icon: null } } });
        expect(parseHeroCache(old, 1001, false)).toBeNull();
        expect(parseHeroCache(old, 1001, true)?.[2].name).toBe("Seven");
    });

    it("rejects garbage", () => {
        expect(parseHeroCache("{", 0, true)).toBeNull();
        expect(parseHeroCache(null, 0, true)).toBeNull();
    });
});
