import { describe, expect, it } from "vitest";
import { parseHeroCache, slimHeroes } from "./heroes";

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
