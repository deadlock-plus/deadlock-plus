import { describe, expect, it } from "vitest";
import { chunk, toProfile } from "./profiles";

describe("toProfile", () => {
    it("maps an API player to a steamid64 profile", () => {
        expect(toProfile({ account_id: 387246372, personaname: "x", avatarmedium: "a", profileurl: "u" })).toEqual({
            steamid64: "76561198347512100",
            name: "x",
            avatar: "a",
            profileUrl: "u",
        });
    });

    it("falls back to Unknown for an empty name", () => {
        expect(toProfile({ account_id: 1, personaname: "" })?.name).toBe("Unknown");
    });
});

describe("chunk", () => {
    it("splits into groups of at most n", () => {
        expect(chunk([1, 2, 3, 4, 5], 2)).toEqual([[1, 2], [3, 4], [5]]);
        expect(chunk([], 3)).toEqual([]);
    });
});
