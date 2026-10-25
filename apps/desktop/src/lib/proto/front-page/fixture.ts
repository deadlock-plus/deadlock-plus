import { formatNumber, t } from "$lib/core/i18n.svelte";
import { formatClock } from "$lib/features/live/live";
import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
import { resolveDetail } from "$lib/features/match-history/api";
import { gradeMatch } from "$lib/features/match-history/mvp";
import { playerNames } from "$lib/features/match-history/names";
import { playerLine, teamLeadLine, valueAt } from "@deadlock-plus/match/deep-dive/chart";
import { laneForObjectiveSlot, laneForPlayerLane, objectiveLaneSlot } from "@deadlock-plus/match/deep-dive/lanes";
import { objectiveNames, type ObjectiveKind } from "@deadlock-plus/match/deep-dive/names";
import {
    allPlayers,
    findPlayer,
    playerBySlot,
    type MatchDetail,
    type MatchPlayer,
    type MatchTeam,
} from "@deadlock-plus/match/detail";
import { rowMode } from "@deadlock-plus/match/list";
import { withNames } from "@deadlock-plus/match/names";
import type { MvpLetter } from "@deadlock-plus/ipc-types/MvpLetter";
import type { MvpPlayerGrade } from "@deadlock-plus/ipc-types/MvpPlayerGrade";

/** The demo's newest match. */
export const MATCH_ID = 990000060;

export interface Line {
    slot: number;
    team: MatchTeam;
    hero: string;
    name: string;
    own: boolean;
    kills: number;
    deaths: number;
    assists: number;
    souls: number;
    damage: number;
    healing: number;
    lastHits: number;
    grade: MvpPlayerGrade | null;
}

export interface TeamBox {
    team: MatchTeam;
    name: string;
    kills: number;
    souls: number;
    won: boolean;
    lines: Line[];
}

export interface Story {
    kind: string;
    score: number;
    kicker: string;
    headline: string;
    body: string;
}

export interface LaneReport {
    colour: string;
    street: string;
    ink: string;
    heroes: Record<MatchTeam, string[]>;
    /** The Hidden King's souls minus The Archmother's in this lane at 10:00. */
    leadAt10: number;
    guardianFalls: { team: MatchTeam; timeS: number }[];
}

export interface ObjectiveMark {
    timeS: number;
    /** Team that took it. */
    by: MatchTeam | null;
    label: string;
    short: string;
    kind: ObjectiveKind | "mid_boss";
}

export interface Weather {
    /** MOCK: the demo has no per-match ping recording, so this series is invented. */
    mock: true;
    avg: number;
    worst: number;
    jitter: number;
    forecast: string;
    points: number[];
}

export interface FrontPage {
    matchId: number;
    startTime: number;
    durationS: number;
    mode: string;
    winner: MatchTeam | null;
    own: Line | null;
    art: string | null;
    artHero: string;
    teams: TeamBox[];
    ratings: Line[];
    stories: Story[];
    lanes: LaneReport[];
    objectives: ObjectiveMark[];
    weather: Weather;
}

export const teamName = (team: MatchTeam) =>
    team === "hidden-king" ? t("match_history.team.hidden_king") : t("match_history.team.archmother");

const other = (team: MatchTeam): MatchTeam => (team === "hidden-king" ? "archmother" : "hidden-king");
export const compact = (n: number) => formatNumber(n, { notation: "compact", maximumFractionDigits: 1 });
export const clock = (s: number) => formatClock(Math.round(s)) ?? "";
export const kda = (l: Pick<Line, "kills" | "deaths" | "assists">) => `${l.kills}/${l.deaths}/${l.assists}`;

const ordinal = (n: number) => {
    const rule = new Intl.PluralRules("en", { type: "ordinal" }).select(n);
    return `${n}${{ one: "st", two: "nd", few: "rd" }[rule as "one" | "two" | "few"] ?? "th"}`;
};
export { ordinal };

const gradeWord = (letter: MvpLetter) => (/^[AEF]/.test(letter) ? `an ${letter}` : `a ${letter}`);

/** MOCK: a deterministic invented ping series, labelled as mock wherever it shows. */
const MOCK_PING = Array.from({ length: 48 }, (_, i) =>
    Math.round(34 + 6 * Math.sin(i / 3) + (i % 7 === 3 ? 19 : 0) + (i % 5)),
);

function weatherOf(points: number[]): Weather {
    const avg = points.reduce((a, b) => a + b, 0) / points.length;
    const diffs = points.slice(1).map((p, i) => Math.abs(p - points[i]));
    const jitter = diffs.reduce((a, b) => a + b, 0) / diffs.length;
    const forecast = avg < 45 ? "Clear skies" : avg < 80 ? "Fair" : avg < 120 ? "Overcast" : "Storms";
    return {
        mock: true,
        avg: Math.round(avg),
        worst: Math.max(...points),
        jitter: Math.round(jitter),
        forecast,
        points,
    };
}

function lineOf(
    p: MatchPlayer,
    heroes: Record<number, Hero>,
    grades: Map<number, MvpPlayerGrade>,
    ownId: number | null,
): Line {
    return {
        slot: p.slot,
        team: p.team,
        hero: heroes[p.heroId]?.name ?? `Hero ${p.heroId}`,
        name: p.name ?? heroes[p.heroId]?.name ?? `Player ${p.slot}`,
        own: ownId !== null && p.accountId === ownId,
        kills: p.kills,
        deaths: p.deaths,
        assists: p.assists,
        souls: p.souls,
        damage: p.playerDamage,
        healing: p.healing,
        lastHits: p.lastHits,
        grade: grades.get(p.slot) ?? null,
    };
}

const top = <T>(xs: T[], key: (x: T) => number) => xs.reduce((a, b) => (key(b) > key(a) ? b : a));

/** Throwaway detectors: each returns a story or nothing; the page sorts by score. */
function stories(detail: MatchDetail, lines: Line[], own: Line | null): Story[] {
    const out: Story[] = [];
    const byslot = new Map(lines.map((l) => [l.slot, l]));
    const winner = detail.winningTeam ?? null;
    const minutes = Math.round(detail.durationS / 60);

    if (winner) {
        const lead = teamLeadLine(detail, "souls");
        const sign = winner === "hidden-king" ? -1 : 1;
        const worst = lead.length ? top(lead, (p) => sign * p.v) : null;
        const deficit = worst ? sign * worst.v : 0;
        if (worst && deficit >= 4000) {
            out.push({
                kind: "comeback",
                score: 60 + deficit / 1000,
                kicker: "Comeback",
                headline: `${teamName(winner)} climb out of a ${compact(deficit)}-soul hole`,
                body: `Down ${formatNumber(deficit)} souls at ${clock(worst.t)}, ${teamName(winner)} turned it around and closed the match at ${clock(detail.durationS)}.`,
            });
        }
        const w = detail.teams.find((x) => x.team === winner);
        const l = detail.teams.find((x) => x.team === other(winner));
        out.push({
            kind: "result",
            score: 50,
            kicker: "Final",
            headline: `${teamName(winner)} win it in ${minutes} minutes`,
            body: `${teamName(winner)} took the match ${w?.score ?? 0} kills to ${l?.score ?? 0} over ${clock(detail.durationS)}.`,
        });
    }

    const graded = lines.filter((l) => l.grade);
    if (graded.length) {
        const mvp = graded.reduce((a, b) => ((b.grade?.place ?? 99) < (a.grade?.place ?? 99) ? b : a));
        const g = mvp.grade as MvpPlayerGrade;
        out.push({
            kind: "mvp",
            score: mvp.own ? 80 : 55,
            kicker: "The critics",
            headline: `${mvp.hero} tops the critics with ${gradeWord(g.letter)}`,
            body: `${mvp.name} went ${kda(mvp)} on ${mvp.hero} and finished 1st of ${graded.length} in the Deadlock+ grades, scoring ${(g.s10 / 10).toFixed(1)}.`,
        });
    }

    if (own) {
        const ratio = (own.kills + own.assists) / Math.max(1, own.deaths);
        out.push({
            kind: "own",
            score: 40 + ratio * 3,
            kicker: "Your match",
            headline: `${own.hero} goes ${kda(own)}`,
            body: `You finished on ${formatNumber(own.souls)} net worth with ${formatNumber(own.damage)} player damage${own.grade ? ` and ${gradeWord(own.grade.letter)}, ${ordinal(own.grade.place)} in the lobby` : ""}.`,
        });
    }

    const dmg = top(lines, (l) => l.damage);
    out.push({
        kind: "damage",
        score: 42,
        kicker: "Firepower",
        headline: `${dmg.hero} deals ${compact(dmg.damage)} damage`,
        body: `${dmg.name} dealt ${formatNumber(dmg.damage)} player damage, the most in the match.`,
    });

    const heal = top(lines, (l) => l.healing);
    if (heal.healing > 0) {
        out.push({
            kind: "healing",
            score: 30,
            kicker: "Medical",
            headline: `${heal.hero} heals ${compact(heal.healing)}`,
            body: `${heal.name} healed ${formatNumber(heal.healing)}, more than anyone else.`,
        });
    }

    const deaths = allPlayers(detail)
        .flatMap((p) => p.deathLog.map((d) => ({ victim: p.slot, ...d })))
        .filter((d) => d.killerSlot >= 1 && d.killerSlot <= 12 && d.killerSlot !== d.victim)
        .sort((a, b) => a.timeS - b.timeS);
    const first = deaths[0];
    if (first) {
        const killer = byslot.get(first.killerSlot);
        const victim = byslot.get(first.victim);
        if (killer && victim) {
            out.push({
                kind: "first-blood",
                score: 34,
                kicker: "First blood",
                headline: `${killer.hero} strikes first at ${clock(first.timeS)}`,
                body: `${killer.name} took down ${victim.hero} for the match's first kill.`,
            });
        }
    }

    const boss = detail.midBoss[0];
    if (boss?.claimedBy) {
        out.push({
            kind: "mid-boss",
            score: 36,
            kicker: "Mid Boss",
            headline: `${teamName(boss.claimedBy)} claim the Mid Boss at ${clock(boss.destroyedS)}`,
            body:
                boss.killedBy && boss.killedBy !== boss.claimedBy
                    ? `${teamName(boss.killedBy)} defeated it; ${teamName(boss.claimedBy)} claimed it.`
                    : `${teamName(boss.claimedBy)} defeated and claimed it.`,
        });
    }

    const names = objectiveNames(detail.objectives);
    const walker = detail.objectives
        .map((o, i) => ({ o, n: names[i] }))
        .filter(({ o, n }) => n.kind === "walker" && o.destroyedS !== undefined && o.destroyedS > 1)
        .sort((a, b) => (a.o.destroyedS ?? 0) - (b.o.destroyedS ?? 0))[0];
    if (walker) {
        const lane = laneForObjectiveSlot(objectiveLaneSlot(walker.o.objectiveId));
        const where = lane ? ` on ${t(`match_history.deep_dive.lanes.${LANE_STREETS[lane.color]}`)}` : "";
        out.push({
            kind: "walker",
            score: 33,
            kicker: "Objectives",
            headline: `First Walker falls at ${clock(walker.o.destroyedS ?? 0)}`,
            body: `${teamName(other(walker.o.team))} brought down ${teamName(walker.o.team)}'s Walker${where}.`,
        });
    }

    const rich = top(lines, (l) => l.souls);
    out.push({
        kind: "net-worth",
        score: 28,
        kicker: "Markets",
        headline: `${rich.hero} closes on ${compact(rich.souls)} net worth`,
        body: `${rich.name} ended the match with the highest net worth: ${formatNumber(rich.souls)}.`,
    });

    const farm = top(lines, (l) => l.lastHits);
    out.push({
        kind: "farm",
        score: 20,
        kicker: "Agriculture",
        headline: `${farm.hero} leads with ${farm.lastHits} last hits`,
        body: `${farm.name} took ${farm.lastHits} last hits, the most in the match.`,
    });

    const fallen = top(lines, (l) => l.deaths);
    out.push({
        kind: "obituary",
        score: 14,
        kicker: "Obituaries",
        headline: `${fallen.hero} fell ${fallen.deaths} times`,
        body: `${fallen.name} fell ${fallen.deaths} times and is survived by ${fallen.assists} assists.`,
    });

    return out.sort((a, b) => b.score - a.score);
}

const LANE_STREETS: Record<string, string> = { yellow: "street_yellow", blue: "street_blue", green: "street_green" };
/** Lane colours mixed down to inks that read on newsprint. */
const LANE_INK: Record<string, string> = { yellow: "#a37a00", blue: "#1f6f86", green: "#3d7a2f" };

function lanes(detail: MatchDetail, heroes: Record<number, Hero>): LaneReport[] {
    const names = objectiveNames(detail.objectives);
    const out: LaneReport[] = [];
    for (const id of [1, 4, 6]) {
        const lane = laneForPlayerLane(id);
        if (!lane) continue;
        const players = allPlayers(detail).filter((p) => p.lane === id);
        if (!players.length) continue;
        const soulsAt = (team: MatchTeam) =>
            players.filter((p) => p.team === team).reduce((n, p) => n + valueAt(playerLine(p, "souls"), 600), 0);
        const guardianFalls = detail.objectives
            .map((o, i) => ({ o, n: names[i] }))
            .filter(
                ({ o, n }) =>
                    n.kind === "guardian" &&
                    o.destroyedS !== undefined &&
                    o.destroyedS > 1 &&
                    laneForObjectiveSlot(objectiveLaneSlot(o.objectiveId))?.id === id,
            )
            .map(({ o }) => ({ team: o.team, timeS: o.destroyedS ?? 0 }))
            .sort((a, b) => a.timeS - b.timeS);
        out.push({
            colour: t(`match_history.deep_dive.lanes.${lane.color}`),
            street: t(`match_history.deep_dive.lanes.${LANE_STREETS[lane.color]}`),
            ink: LANE_INK[lane.color],
            heroes: {
                "hidden-king": players
                    .filter((p) => p.team === "hidden-king")
                    .map((p) => heroes[p.heroId]?.name ?? "?"),
                archmother: players.filter((p) => p.team === "archmother").map((p) => heroes[p.heroId]?.name ?? "?"),
            },
            leadAt10: Math.round(soulsAt("hidden-king") - soulsAt("archmother")),
            guardianFalls,
        });
    }
    return out;
}

const SHORT: Record<ObjectiveKind, string> = {
    guardian: "G",
    walker: "W",
    base_guardian: "BG",
    shrine: "S",
    patron: "P",
    core: "C",
    unknown: "?",
};

function objectives(detail: MatchDetail): ObjectiveMark[] {
    const names = objectiveNames(detail.objectives);
    const marks: ObjectiveMark[] = detail.objectives.flatMap((o, i) => {
        if (o.destroyedS === undefined || o.destroyedS <= 1) return [];
        const n = names[i];
        const label = t(`match_history.deep_dive.objectives.kind_${n.kind}`);
        return [{ timeS: o.destroyedS, by: other(o.team), label, short: SHORT[n.kind], kind: n.kind }];
    });
    for (const m of detail.midBoss) {
        marks.push({ timeS: m.destroyedS, by: m.claimedBy ?? null, label: "Mid Boss", short: "MB", kind: "mid_boss" });
    }
    return marks.sort((a, b) => a.timeS - b.timeS);
}

async function namedDetail(matchId: number): Promise<MatchDetail> {
    const detail = await resolveDetail(matchId);
    if (!detail) throw new Error(`demo match ${matchId} is missing`);
    try {
        return withNames(detail, await playerNames.resolve(allPlayers(detail).map((p) => p.accountId)));
    } catch {
        return detail;
    }
}

export async function loadFrontPage(ownAccountId: number | null, matchId = MATCH_ID): Promise<FrontPage> {
    const detail = await namedDetail(matchId);
    const [heroes, grade] = await Promise.all([
        loadHeroes([...new Set(allPlayers(detail).map((p) => p.heroId))]),
        gradeMatch(matchId).catch(() => null),
    ]);
    const grades = new Map((grade?.players ?? []).map((g) => [g.playerSlot, g]));
    const lines = allPlayers(detail).map((p) => lineOf(p, heroes, grades, ownAccountId));
    const ownPlayer = ownAccountId === null ? undefined : findPlayer(detail, [ownAccountId]);
    const own = lines.find((l) => l.own) ?? null;
    const ownTeam = own?.team ?? "hidden-king";
    const teams = [ownTeam, other(ownTeam)].map((team): TeamBox => {
        const ls = lines.filter((l) => l.team === team).sort((a, b) => b.souls - a.souls);
        return {
            team,
            name: teamName(team),
            kills: ls.reduce((n, l) => n + l.kills, 0),
            souls: ls.reduce((n, l) => n + l.souls, 0),
            won: detail.winningTeam === team,
            lines: ls,
        };
    });
    const ratings = lines.filter((l) => l.grade).sort((a, b) => (a.grade?.place ?? 0) - (b.grade?.place ?? 0));
    const artPlayer = ownPlayer ?? playerBySlot(detail, ratings[0]?.slot ?? 1);
    const artHero = artPlayer ? heroes[artPlayer.heroId] : undefined;

    return {
        matchId,
        startTime: detail.startTime,
        durationS: detail.durationS,
        mode: t(`match_history.detail.mode.${rowMode(detail)}`),
        winner: detail.winningTeam ?? null,
        own,
        art: artHero?.portrait ?? null,
        artHero: artHero?.name ?? "",
        teams,
        ratings,
        stories: stories(detail, lines, own),
        lanes: lanes(detail, heroes),
        objectives: objectives(detail),
        weather: weatherOf(MOCK_PING),
    };
}
