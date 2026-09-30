import { describe, expect, it } from "vitest";
import {
    isForcedExit,
    newestFirst,
    shouldOpenNotes,
    seenFlagTiming,
    visibleReleases,
    WHATS_NEW_ROUTE,
    WHATS_NEW_UPDATE_ROUTE,
} from "./whats-new";

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

describe("shouldOpenNotes", () => {
    const base = { noteCount: 2, onboardingPending: false, pathname: "/" };
    it("opens when there are notes and nothing else is in the way", () => {
        expect(shouldOpenNotes(base)).toBe(true);
    });
    it("stays away when there are no notes", () => {
        expect(shouldOpenNotes({ ...base, noteCount: 0 })).toBe(false);
    });
    it("yields to a pending onboarding", () => {
        expect(shouldOpenNotes({ ...base, onboardingPending: true })).toBe(false);
    });
    it("yields when already on the onboarding route", () => {
        expect(shouldOpenNotes({ ...base, pathname: "/onboarding" })).toBe(false);
        expect(shouldOpenNotes({ ...base, pathname: "/onboarding/x" })).toBe(false);
    });
    it("does not reopen when already on the page", () => {
        expect(shouldOpenNotes({ ...base, pathname: "/whats-new" })).toBe(false);
    });
});

describe("visibleReleases", () => {
    const e = (version: string) => ({ version, date: null, sections: [] });
    const history = [e("0.1.0"), e("0.2.0"), e("0.3.0")];
    const entries = [e("0.3.0")];
    it("shows only the new releases after an update", () => {
        expect(visibleReleases({ forced: true, showAll: false, entries, history }).map((x) => x.version)).toEqual([
            "0.3.0",
        ]);
    });
    it("shows the whole history, newest first, when opened by hand", () => {
        expect(visibleReleases({ forced: false, showAll: false, entries, history }).map((x) => x.version)).toEqual([
            "0.3.0",
            "0.2.0",
            "0.1.0",
        ]);
    });
    it("shows the whole history when the user asks for it", () => {
        expect(visibleReleases({ forced: true, showAll: true, entries, history })).toHaveLength(3);
    });
    it("falls back to the history when an update has no new entries", () => {
        expect(visibleReleases({ forced: true, showAll: false, entries: [], history })).toHaveLength(3);
    });
});
