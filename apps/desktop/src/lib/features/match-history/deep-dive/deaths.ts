import { allPlayers, type MatchDetail, type MatchPlayer, type MatchTeam, type Position } from "../detail";

export interface DeathRow {
    timeS: number;
    killerSlot: number;
    /** Absent when the killer's slot is not a player in this match (a non-hero killer, or a trimmed roster). */
    killerHeroId?: number;
    killerTeam?: MatchTeam;
    respawnS?: number;
    position?: Position;
}

export function deathRows(detail: MatchDetail, player: MatchPlayer): DeathRow[] {
    const bySlot = new Map(allPlayers(detail).map((p) => [p.slot, p]));
    return [...player.deathLog]
        .sort((a, b) => a.timeS - b.timeS)
        .map((d) => ({
            timeS: d.timeS,
            killerSlot: d.killerSlot,
            killerHeroId: bySlot.get(d.killerSlot)?.heroId,
            killerTeam: bySlot.get(d.killerSlot)?.team,
            respawnS: d.respawnS,
            position: d.position,
        }));
}
