import { describe, expect, it } from "vitest";
import {
    addMatch,
    finishedMatch,
    MAX_MATCHES,
    MIN_MATCH_MS,
    parseMatches,
    stepMatch,
    type MatchRecord,
} from "./match-history";

const rec = (startedAt: number): MatchRecord => ({
    id: String(startedAt),
    startedAt,
    endedAt: startedAt + 1,
    avg: 40,
    worst: 90,
    samples: 10,
    server: null,
});

describe("stepMatch", () => {
    it("opens when the match starts and keeps the first start time", () => {
        const a = stepMatch(null, "inMatch", 100);
        expect(a).toEqual({ open: { startedAt: 100 }, closed: null });
        expect(stepMatch(a.open, "inMatch", 200)).toEqual({ open: { startedAt: 100 }, closed: null });
    });

    it("does not count pregame or menus as in match", () => {
        for (const p of ["pregame", "queuing", "menus", "gameClosed", "postMatch"] as const) {
            expect(stepMatch(null, p, 100)).toEqual({ open: null, closed: null });
        }
    });

    it("closes on any phase that is not in match", () => {
        for (const p of ["postMatch", "menus", "gameClosed", "pregame"] as const) {
            expect(stepMatch({ startedAt: 100 }, p, 900)).toEqual({ open: null, closed: { startedAt: 100 } });
        }
    });

    it("ignores an unknown state", () => {
        expect(stepMatch({ startedAt: 100 }, undefined, 900).open).toEqual({ startedAt: 100 });
    });
});

describe("finishedMatch", () => {
    const summary = { avg: 41.26, worst: 120, samples: 300 };

    it("builds a record with rounded ping", () => {
        const end = 100 + MIN_MATCH_MS;
        expect(finishedMatch({ startedAt: 100, server: "Frankfurt" }, end, summary)).toEqual({
            id: "100",
            startedAt: 100,
            endedAt: end,
            avg: 41.3,
            worst: 120,
            samples: 300,
            server: "Frankfurt",
        });
    });

    it("drops blips and matches with no ping data", () => {
        expect(finishedMatch({ startedAt: 100 }, 100 + MIN_MATCH_MS - 1, summary)).toBeNull();
        expect(finishedMatch({ startedAt: 100 }, 100 + MIN_MATCH_MS, null)).toBeNull();
    });
});

describe("addMatch", () => {
    it("puts the newest first and caps the list", () => {
        let list: MatchRecord[] = [];
        for (let i = 1; i <= MAX_MATCHES + 5; i++) list = addMatch(list, rec(i));
        expect(list).toHaveLength(MAX_MATCHES);
        expect(list[0].startedAt).toBe(MAX_MATCHES + 5);
    });

    it("replaces a record with the same id", () => {
        expect(addMatch([rec(1)], { ...rec(1), avg: 99 })).toEqual([{ ...rec(1), avg: 99 }]);
    });
});

describe("parseMatches", () => {
    it("keeps valid records and drops junk", () => {
        expect(parseMatches([rec(1), { id: "x" }, null, "no"])).toEqual([rec(1)]);
        expect(parseMatches(undefined)).toEqual([]);
    });
});
