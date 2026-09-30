import { platformName, type Platform } from "$lib/core/platform";

export const ONBOARDING_VERSION = 1;

export interface GameStatus {
    found: boolean;
    path: string | null;
}

export type StepId = "welcome" | "features" | "permissions" | "steam" | "sharing" | "extras" | "done";

export interface Step {
    id: StepId;
    title: string;
}

export interface FeatureBlurb {
    id: string;
    blurb: string;
}

export interface FeatureGroup {
    title: string;
    items: FeatureBlurb[];
}

export type IngestChoice = "share" | "decline";

export const FEATURE_GROUPS: FeatureGroup[] = [
    {
        title: "Play",
        items: [
            { id: "server-picker", blurb: "Block the server regions you don't want." },
            { id: "connection", blurb: "Live ping and packet loss for your match." },
        ],
    },
    {
        title: "Progress",
        items: [
            { id: "stats", blurb: "Winrate and hero numbers from your match history." },
            { id: "rank", blurb: "Your rank over time, from your match history." },
            { id: "sessions", blurb: "How your results change as sessions get longer." },
        ],
    },
    {
        title: "News",
        items: [{ id: "alerts", blurb: "Patch notes and announcements, searchable." }],
    },
    {
        title: "Housekeeping",
        items: [
            { id: "voice-bans", blurb: "See, add and unmute your muted players." },
            { id: "demos", blurb: "Browse your saved replays and spot outdated ones." },
            { id: "storage", blurb: "See what Deadlock uses on disk and clear what is safe." },
            { id: "performance", blurb: "Scan addons for scripts that can hurt frametimes." },
        ],
    },
];

export function needsOnboarding(stored: number | null | undefined): boolean {
    return typeof stored !== "number" || stored < ONBOARDING_VERSION;
}

export function isReturningUser(legacyDone: boolean | null | undefined): boolean {
    return legacyDone === true;
}

export function gameStatus(gameDir: string | null | undefined): GameStatus {
    return gameDir ? { found: true, path: gameDir } : { found: false, path: null };
}

export function stepsFor(platform: Platform): Step[] {
    const permissions =
        platform === "windows" ? "Why Windows asks for permission" : `Running on ${platformName(platform)}`;
    return [
        { id: "welcome", title: "Welcome to Deadlock+" },
        { id: "features", title: "What Deadlock+ does" },
        { id: "permissions", title: permissions },
        { id: "steam", title: "Your Steam account" },
        { id: "sharing", title: "Help the Deadlock community?" },
        { id: "extras", title: "Optional extras" },
        { id: "done", title: "You're set" },
    ];
}

export function nextStep(index: number, count: number): number {
    return Math.min(index + 1, count - 1);
}

export function previousStep(index: number): number {
    return Math.max(index - 1, 0);
}

export async function finishOnboarding(write: (version: number) => Promise<void>): Promise<void> {
    try {
        await write(ONBOARDING_VERSION);
    } catch {
        // It will just show again next launch.
    }
}

/** `null` means the user skipped the step: consent stays unanswered. */
export function ingestDecision(choice: IngestChoice | null): boolean | null {
    if (choice === null) return null;
    return choice === "share";
}
