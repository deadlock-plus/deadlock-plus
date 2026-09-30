import { describe, expect, it } from "vitest";
import { isForcedExit, newestFirst, seenFlagTiming, WHATS_NEW_ROUTE, WHATS_NEW_UPDATE_ROUTE } from "./whats-new";

describe("seenFlagTiming", () => {
    it("writes nothing when the flag already matches", () => {
        expect(seenFlagTiming({ lastSeen: "1.0.0", appVersion: "1.0.0", noteCount: 0 })).toBe("never");
        expect(seenFlagTiming({ lastSeen: "1.0.0", appVersion: "1.0.0", noteCount: 2 })).toBe("never");
    });
    it("writes at once when there is nothing to show", () => {
        expect(seenFlagTiming({ lastSeen: null, appVersion: "1.0.0", noteCount: 0 })).toBe("now");
        expect(seenFlagTiming({ lastSeen: "0.9.0", appVersion: "1.0.0", noteCount: 0 })).toBe("now");
    });
    it("waits for the user when there are notes to read", () => {
        expect(seenFlagTiming({ lastSeen: "0.9.0", appVersion: "1.0.0", noteCount: 1 })).toBe("on-seen");
    });
});

describe("isForcedExit", () => {
    it("is true only for the update source", () => {
        expect(isForcedExit(new URLSearchParams("from=update"))).toBe(true);
        expect(isForcedExit(new URLSearchParams(""))).toBe(false);
        expect(isForcedExit(new URLSearchParams("from=settings"))).toBe(false);
    });
    it("matches the update route", () => {
        const [, query] = WHATS_NEW_UPDATE_ROUTE.split("?");
        expect(WHATS_NEW_UPDATE_ROUTE.startsWith(WHATS_NEW_ROUTE)).toBe(true);
        expect(isForcedExit(new URLSearchParams(query))).toBe(true);
    });
});

describe("newestFirst", () => {
    it("sorts by version descending without mutating", () => {
        const e = (version: string) => ({ version, date: null, sections: [] });
        const input = [e("0.1.0"), e("0.10.0"), e("0.2.0")];
        expect(newestFirst(input).map((x) => x.version)).toEqual(["0.10.0", "0.2.0", "0.1.0"]);
        expect(input[0].version).toBe("0.1.0");
    });
});
