import { describe, expect, it } from "vitest";
import {
    BOARD_LINGER_MS,
    boardVisible,
    formatClock,
    formatCompact,
    orderTeams,
    soulsLead,
    formatPercent,
    heroInitials,
    killParticipation,
    postMatchStart,
    rankView,
    soulsPerMinute,
    stateLine,
    teamTotals,
} from "./live";
import type { LiveTeam } from "$lib/generated/types/LiveTeam";
import type { LivePlayer } from "$lib/generated/types/LivePlayer";
import type { LivePhase } from "$lib/generated/types/LivePhase";

describe("stateLine", () => {
    it("shows nothing when unsupported", () => {
        expect(stateLine("unsupported")).toBeNull();
    });

    it("gives every other phase a distinct line", () => {
        const phases: LivePhase[] = ["gameClosed", "menus", "queuing", "pregame", "inMatch", "postMatch"];
        const lines = phases.map((p) => stateLine(p));
        expect(lines.every((l) => l !== null)).toBe(true);
        expect(new Set(lines.map((l) => l?.key)).size).toBe(phases.length);
    });
});

const player = (over: Partial<LivePlayer> = {}): LivePlayer => ({
    key: 0,
    side: "amber",
    name: "A",
    heroId: 1,
    rank: 52,
    souls: 1000,
    kills: 1,
    deaths: 2,
    assists: 3,
    heroDamage: 100,
    objectiveDamage: 10,
    healing: 0,
    isYou: false,
    ...over,
});

describe("soulsPerMinute", () => {
    it("divides souls by minutes played", () => {
        expect(soulsPerMinute(30000, 1200)).toBe(1500);
    });

    it("is null before the first minute", () => {
        expect(soulsPerMinute(500, 59)).toBeNull();
        expect(soulsPerMinute(500, 60)).toBe(500);
    });

    it("is null without a clock or souls", () => {
        expect(soulsPerMinute(500, null)).toBeNull();
        expect(soulsPerMinute(null, 600)).toBeNull();
    });
});

describe("killParticipation", () => {
    it("is (kills + assists) over team kills", () => {
        expect(killParticipation(4, 6, 20)).toBe(0.5);
    });

    it("is null when team kills is zero", () => {
        expect(killParticipation(0, 0, 0)).toBeNull();
    });

    it("is null when anything is unread", () => {
        expect(killParticipation(null, 1, 10)).toBeNull();
        expect(killParticipation(1, null, 10)).toBeNull();
        expect(killParticipation(1, 1, null)).toBeNull();
    });

    it("formats as a whole percent", () => {
        expect(formatPercent(0.684)).toBe("68%");
        expect(formatPercent(null)).toBeNull();
    });
});

describe("formatCompact", () => {
    it("keeps small numbers whole", () => {
        expect(formatCompact(0)).toBe("0");
        expect(formatCompact(999)).toBe("999");
    });

    it("shortens thousands", () => {
        expect(formatCompact(31240)).toBe("31.2k");
        expect(formatCompact(1000)).toBe("1.0k");
    });

    it("is null for unread values", () => {
        expect(formatCompact(null)).toBeNull();
    });
});

describe("rankView", () => {
    it("splits the packed tier and subrank", () => {
        expect(rankView(52)).toEqual({ tier: 5, sub: 2 });
        expect(rankView(113)).toEqual({ tier: 11, sub: 3 });
    });

    it("is null when unranked or unread", () => {
        expect(rankView(0)).toBeNull();
        expect(rankView(null)).toBeNull();
    });
});

describe("heroInitials", () => {
    it("takes up to two letters", () => {
        expect(heroInitials("Lady Geist")).toBe("LG");
        expect(heroInitials("Seven")).toBe("SE");
    });

    it("falls back to a question mark", () => {
        expect(heroInitials(null)).toBe("?");
        expect(heroInitials("")).toBe("?");
    });
});

describe("teamTotals", () => {
    it("sums what is read and skips nulls", () => {
        const t = teamTotals([
            player({ kills: 2, deaths: 1, assists: 3, heroDamage: 100, objectiveDamage: 5, healing: 10 }),
            player({ kills: null, deaths: 4, assists: 1, heroDamage: null, objectiveDamage: 5, healing: null }),
        ]);
        expect(t).toMatchObject({ kills: 2, deaths: 5, assists: 4, heroDamage: 100, objectiveDamage: 10, healing: 10 });
    });

    it("is null for a column with nothing read", () => {
        const t = teamTotals([player({ healing: null }), player({ healing: null })]);
        expect(t.healing).toBeNull();
    });
});

describe("boardVisible", () => {
    it("shows during pregame and the match", () => {
        expect(boardVisible("pregame", null)).toBe(true);
        expect(boardVisible("inMatch", null)).toBe(true);
    });

    it("hides while queuing, even inside the window", () => {
        expect(boardVisible("queuing", 1000)).toBe(false);
    });

    it("lingers after the match ends", () => {
        expect(boardVisible("postMatch", 0)).toBe(true);
        expect(boardVisible("postMatch", BOARD_LINGER_MS - 1)).toBe(true);
        expect(boardVisible("postMatch", BOARD_LINGER_MS)).toBe(false);
    });

    it("keeps lingering back in the menus, then hides", () => {
        expect(boardVisible("menus", 5000)).toBe(true);
        expect(boardVisible("gameClosed", 5000)).toBe(true);
        expect(boardVisible("menus", BOARD_LINGER_MS)).toBe(false);
    });

    it("is hidden in the menus when no match just ended", () => {
        expect(boardVisible("menus", null)).toBe(false);
        expect(boardVisible("unsupported", null)).toBe(false);
    });
});

describe("postMatchStart", () => {
    it("starts when the phase becomes postMatch", () => {
        expect(postMatchStart("postMatch", null, 100)).toBe(100);
    });

    it("keeps the start while postMatch or the menus follow", () => {
        expect(postMatchStart("postMatch", 100, 500)).toBe(100);
        expect(postMatchStart("menus", 100, 500)).toBe(100);
        expect(postMatchStart("gameClosed", 100, 500)).toBe(100);
    });

    it("clears when a new match starts", () => {
        for (const p of ["queuing", "pregame", "inMatch", "unsupported"] as const) {
            expect(postMatchStart(p, 100, 500)).toBeNull();
        }
    });

    it("stays unset in the menus with no finished match", () => {
        expect(postMatchStart("menus", null, 500)).toBeNull();
    });
});

describe("formatClock", () => {
    it("formats minutes and seconds", () => {
        expect(formatClock(0)).toBe("0:00");
        expect(formatClock(65)).toBe("1:05");
        expect(formatClock(2285)).toBe("38:05");
    });

    it("adds hours past an hour", () => {
        expect(formatClock(3725)).toBe("1:02:05");
    });

    it("is null without a clock", () => {
        expect(formatClock(null)).toBeNull();
    });
});

const team = (side: "amber" | "sapphire", souls: number): LiveTeam => ({ side, souls, players: [] });

describe("soulsLead", () => {
    it("is the signed percent a team is ahead of the other team", () => {
        expect(soulsLead(110, 210)).toBe("+10%");
        expect(soulsLead(90, 190)).toBe("-10%");
        expect(soulsLead(100, 200)).toBe("0%");
    });

    it("is null when the other team has no souls or a value is unread", () => {
        expect(soulsLead(100, 100)).toBeNull();
        expect(soulsLead(null, 100)).toBeNull();
    });
});

describe("orderTeams", () => {
    it("puts your team first", () => {
        const t = [team("amber", 1), team("sapphire", 2)];
        expect(orderTeams(t, "sapphire").map((x) => x.side)).toEqual(["sapphire", "amber"]);
        expect(orderTeams(t, null).map((x) => x.side)).toEqual(["amber", "sapphire"]);
    });
});
