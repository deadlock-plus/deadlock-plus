import type { LivePhase } from "$lib/generated/types/LivePhase";

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
