import { describe, expect, it } from "vitest";
import {
    countByStatus,
    deleteCopy,
    formatBytes,
    formatDuration,
    gbToMb,
    matchesTotal,
    mbToGb,
    ruleLabel,
    unpinnedNames,
    withAllRules,
    type CleanupRule,
    matchResult,
    myPlayer,
    statlockerMatchUrl,
    statusInfo,
    totalSize,
    type Demo,
    type MatchSummary,
} from "./demos";

const demo = (over: Partial<Demo>): Demo => ({
    matchId: 1,
    fileName: "1.dem",
    size: 0,
    modifiedMs: 0,
    status: "complete",
    buildNum: 10854,
    ...over,
});

describe("formatBytes", () => {
    it("uses the largest whole unit with one decimal", () => {
        expect(formatBytes(0)).toBe("0 B");
        expect(formatBytes(512)).toBe("512 B");
        expect(formatBytes(1536)).toBe("1.5 KB");
        expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
        expect(formatBytes(19.4 * 1024 ** 3)).toBe("19.4 GB");
    });
});

describe("statusInfo", () => {
    it("labels outdated replays as possibly unplayable, not broken", () => {
        expect(statusInfo("outdated").label).toBe("Older build");
        expect(statusInfo("outdated").hint).toMatch(/may not play/i);
    });

    it("labels every status", () => {
        for (const s of ["complete", "partial", "outdated", "unknown"] as const) {
            expect(statusInfo(s).label.length).toBeGreaterThan(0);
        }
    });
});

describe("totals", () => {
    it("sums sizes", () => {
        expect(totalSize([demo({ size: 10 }), demo({ size: 32 })])).toBe(42);
        expect(totalSize([])).toBe(0);
    });

    it("counts by status", () => {
        const list = [demo({}), demo({ status: "partial" }), demo({ status: "partial" }), demo({ status: "outdated" })];
        expect(countByStatus(list)).toEqual({ complete: 1, partial: 2, outdated: 1, unknown: 0 });
    });
});

const summary: MatchSummary = {
    matchId: 7,
    startTime: 1789947499,
    durationS: 2070,
    winningTeam: 1,
    players: [
        { accountId: 100, heroId: 65, team: 1, kills: 6, deaths: 8, assists: 20 },
        { accountId: 200, heroId: 19, team: 0, kills: 8, deaths: 15, assists: 17 },
    ],
};

describe("match helpers", () => {
    it("builds the Statlocker match summary link", () => {
        expect(statlockerMatchUrl(106837748)).toBe("https://statlocker.gg/match/106837748/summary");
    });

    it("finds the first listed account that is in the match", () => {
        expect(myPlayer(summary, [200])?.heroId).toBe(19);
        expect(myPlayer(summary, [999, 200, 100])?.heroId).toBe(19);
        expect(myPlayer(summary, [100, 200])?.heroId).toBe(65);
        expect(myPlayer(summary, [999])).toBeNull();
        expect(myPlayer(summary, [])).toBeNull();
    });

    it("reports win or loss only for a player in the match", () => {
        expect(matchResult(summary, [100])).toBe("win");
        expect(matchResult(summary, [999, 200])).toBe("loss");
        expect(matchResult(summary, [999])).toBeNull();
    });

    it("formats durations as m:ss and h:mm:ss", () => {
        expect(formatDuration(0)).toBe("0:00");
        expect(formatDuration(65)).toBe("1:05");
        expect(formatDuration(2070)).toBe("34:30");
        expect(formatDuration(3725)).toBe("1:02:05");
    });
});

describe("deleteCopy", () => {
    const GB = 1024 ** 3;
    const base = { count: 1, totalBytes: 2 * GB, binFreeBytes: 50 * GB };

    it("offers the Recycle Bin when it has room", () => {
        const c = deleteCopy({ ...base, recycle: "available" });
        expect(c.title).toBe("Delete 1 replay?");
        expect(c.canRecycle).toBe(true);
        expect(c.notice).toBeNull();
    });

    it("switches to a too-large message for one file", () => {
        const c = deleteCopy({ ...base, recycle: "tooLarge", binFreeBytes: 1 * GB });
        expect(c.canRecycle).toBe(false);
        expect(c.notice).toMatch(/^This file is too large to move to the Recycle Bin/);
        expect(c.notice).toContain("permanent");
    });

    it("uses plural wording for several files", () => {
        const c = deleteCopy({ ...base, count: 3, recycle: "tooLarge" });
        expect(c.title).toBe("Delete 3 replays?");
        expect(c.notice).toMatch(/^These files are too large to move to the Recycle Bin/);
    });

    it("explains a disabled Recycle Bin", () => {
        const c = deleteCopy({ ...base, recycle: "disabled" });
        expect(c.canRecycle).toBe(false);
        expect(c.notice).toMatch(/turned off/i);
    });
});

describe("unpinnedNames", () => {
    it("drops pinned replays, including partial files, from a selection", () => {
        const demos = [
            demo({ matchId: 1, fileName: "1.dem" }),
            demo({ matchId: 2, fileName: "2.dem.partial", status: "partial" }),
            demo({ matchId: 3, fileName: "3.dem" }),
        ];
        expect(unpinnedNames(demos, new Set([2]))).toEqual(["1.dem", "3.dem"]);
    });
});

describe("withAllRules", () => {
    it("adds every missing rule kind as disabled and keeps saved values, in a fixed order", () => {
        const saved: CleanupRule[] = [{ id: 1, enabled: true, kind: "olderThanDays", days: 90 }];
        const all = withAllRules(saved);
        expect(all.map((r) => r.kind)).toEqual(["olderThanDays", "largerThanMb", "outdated", "partial"]);
        expect(all[0]).toEqual({ id: 1, enabled: true, kind: "olderThanDays", days: 90 });
        expect(all.slice(1).every((r) => !r.enabled)).toBe(true);
        expect(new Set(all.map((r) => r.id)).size).toBe(4);
    });

    it("starts with everything off when nothing was saved", () => {
        expect(withAllRules([]).every((r) => !r.enabled)).toBe(true);
    });
});

describe("ruleLabel", () => {
    it("describes each rule in plain words", () => {
        expect(ruleLabel({ id: 1, enabled: true, kind: "olderThanDays", days: 30 })).toBe("Older than 30 days");
        expect(ruleLabel({ id: 1, enabled: true, kind: "olderThanDays", days: 1 })).toBe("Older than 1 day");
        expect(ruleLabel({ id: 2, enabled: true, kind: "largerThanMb", mb: 2048 })).toBe("Larger than 2 GB");
        expect(ruleLabel({ id: 3, enabled: true, kind: "outdated" })).toBe("Older game build");
        expect(ruleLabel({ id: 4, enabled: true, kind: "partial" })).toBe("Unfinished downloads");
    });
});

describe("size limit conversion", () => {
    it("converts between GB and whole MB and never yields zero", () => {
        expect(gbToMb(2)).toBe(2048);
        expect(gbToMb(0.5)).toBe(512);
        expect(gbToMb(0)).toBe(1);
        expect(gbToMb(Number.NaN)).toBe(1);
        expect(mbToGb(1536)).toBe(1.5);
    });
});

describe("matchesTotal", () => {
    it("counts files and adds their sizes", () => {
        expect(matchesTotal([{ size: 10 }, { size: 5 }])).toEqual({ count: 2, bytes: 15 });
        expect(matchesTotal([])).toEqual({ count: 0, bytes: 0 });
    });
});
