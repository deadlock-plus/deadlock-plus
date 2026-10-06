import type { LivePhase } from "$lib/generated/types/LivePhase";
import type { LiveSide } from "$lib/generated/types/LiveSide";
import type { LiveTeam } from "$lib/generated/types/LiveTeam";
import type { LivePlayer } from "$lib/generated/types/LivePlayer";
import { badgeParts, type BadgeParts } from "$lib/features/stats/rank";

export interface StateLine {
    key: string;
}

const LINES: Record<Exclude<LivePhase, "unsupported">, StateLine> = {
    gameClosed: { key: "live.state.game_closed" },
    menus: { key: "live.state.menus" },
    queuing: { key: "live.state.queuing" },
    pregame: { key: "live.state.pregame" },
    inMatch: { key: "live.state.in_match" },
    postMatch: { key: "live.state.post_match" },
};

export function stateLine(phase: LivePhase): StateLine | null {
    return phase === "unsupported" ? null : LINES[phase];
}

export const BOARD_LINGER_MS = 20_000;
const MIN_CLOCK_SECS = 60;

export function soulsPerMinute(souls: number | null, clockSecs: number | null): number | null {
    if (souls === null || clockSecs === null || clockSecs < MIN_CLOCK_SECS) return null;
    return souls / (clockSecs / 60);
}

export function killParticipation(
    kills: number | null,
    assists: number | null,
    teamKills: number | null,
): number | null {
    if (kills === null || assists === null || teamKills === null || teamKills <= 0) return null;
    return (kills + assists) / teamKills;
}

export function formatPercent(fraction: number | null): string | null {
    return fraction === null ? null : `${Math.round(fraction * 100)}%`;
}

/** How far a team's souls are ahead of (+) or behind (-) the other team, as a signed whole percent. */
export function soulsLead(teamSouls: number | null, totalSouls: number): string | null {
    if (teamSouls === null) return null;
    const other = totalSouls - teamSouls;
    if (other <= 0) return null;
    const pct = Math.round(((teamSouls - other) / other) * 100);
    return pct > 0 ? `+${pct}%` : `${pct}%`;
}

export function formatCompact(value: number | null): string | null {
    if (value === null) return null;
    return value >= 1000 ? `${(value / 1000).toFixed(1)}k` : String(Math.round(value));
}

export function rankView(rank: number | null): BadgeParts | null {
    return badgeParts(rank);
}

export function heroInitials(name: string | null): string {
    const words = (name ?? "").split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w));
    const letters = (w: string) => w.match(/[\p{L}\p{N}]/gu) ?? [];
    const picked =
        words.length >= 2 ? [letters(words[0])[0], letters(words[1])[0]] : letters(words[0] ?? "").slice(0, 2);
    return picked.length ? picked.join("").toUpperCase() : "?";
}

export interface TeamTotals {
    souls: number | null;
    kills: number | null;
    deaths: number | null;
    assists: number | null;
    heroDamage: number | null;
    objectiveDamage: number | null;
    healing: number | null;
}

type Column = keyof TeamTotals;

function sumColumn(players: LivePlayer[], column: Column): number | null {
    let total: number | null = null;
    for (const p of players) {
        const v = p[column];
        if (v !== null) total = (total ?? 0) + v;
    }
    return total;
}

export function teamTotals(players: LivePlayer[]): TeamTotals {
    return {
        souls: sumColumn(players, "souls"),
        kills: sumColumn(players, "kills"),
        deaths: sumColumn(players, "deaths"),
        assists: sumColumn(players, "assists"),
        heroDamage: sumColumn(players, "heroDamage"),
        objectiveDamage: sumColumn(players, "objectiveDamage"),
        healing: sumColumn(players, "healing"),
    };
}

/** `sincePostMatchMs` is how long ago the phase became `postMatch`, or `null` if it has not. */
export function boardVisible(phase: LivePhase, sincePostMatchMs: number | null): boolean {
    switch (phase) {
        case "pregame":
        case "inMatch":
            return true;
        case "postMatch":
        case "menus":
        case "gameClosed":
            return sincePostMatchMs !== null && sincePostMatchMs < BOARD_LINGER_MS;
        default:
            return false;
    }
}

export function postMatchStart(phase: LivePhase, current: number | null, now: number): number | null {
    switch (phase) {
        case "postMatch":
            return current ?? now;
        case "menus":
        case "gameClosed":
            return current;
        default:
            return null;
    }
}

export function formatClock(secs: number | null): string | null {
    if (secs === null) return null;
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const ss = String(secs % 60).padStart(2, "0");
    return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
}

export function orderTeams(teams: LiveTeam[], yourSide: LiveSide | null): LiveTeam[] {
    const first = yourSide ?? "amber";
    return [...teams].sort((a, b) => Number(b.side === first) - Number(a.side === first));
}

export function steamProfileUrl(steamId: string): string {
    return `https://steamcommunity.com/profiles/${steamId}`;
}
