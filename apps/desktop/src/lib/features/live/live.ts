import type { LivePhase } from "$lib/generated/types/LivePhase";

export interface StateLine {
    key: string;
    settingsLink: boolean;
}

const LINES: Record<Exclude<LivePhase, "unsupported">, StateLine> = {
    readingOff: { key: "live.state.reading_off", settingsLink: true },
    gameClosed: { key: "live.state.game_closed", settingsLink: false },
    menus: { key: "live.state.menus", settingsLink: false },
    queuing: { key: "live.state.queuing", settingsLink: false },
    pregame: { key: "live.state.pregame", settingsLink: false },
    inMatch: { key: "live.state.in_match", settingsLink: false },
    postMatch: { key: "live.state.post_match", settingsLink: false },
};

export function stateLine(phase: LivePhase): StateLine | null {
    return phase === "unsupported" ? null : LINES[phase];
}
