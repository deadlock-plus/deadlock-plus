import { compareVersions, type ChangelogEntry } from "./changelog";

export const WHATS_NEW_ROUTE = "/whats-new";
export const WHATS_NEW_UPDATE_ROUTE = `${WHATS_NEW_ROUTE}?from=update`;

export type SeenFlagTiming = "never" | "now" | "on-seen";

/** With notes to show, the flag waits until the user has seen them so an interrupted session shows them again. */
export function seenFlagTiming(input: {
    lastSeen: string | null;
    appVersion: string;
    noteCount: number;
}): SeenFlagTiming {
    if (input.lastSeen === input.appVersion) return "never";
    return input.noteCount > 0 ? "on-seen" : "now";
}

export function isForcedExit(params: URLSearchParams): boolean {
    return params.get("from") === "update";
}

export function newestFirst(entries: ChangelogEntry[]): ChangelogEntry[] {
    return [...entries].sort((a, b) => compareVersions(b.version, a.version));
}

/** After an update the page shows just what changed; the full history is one click away. */
export function visibleReleases(input: {
    forced: boolean;
    showAll: boolean;
    entries: ChangelogEntry[];
    history: ChangelogEntry[];
}): ChangelogEntry[] {
    const onlyNew = input.forced && !input.showAll && input.entries.length > 0;
    return newestFirst(onlyNew ? input.entries : input.history);
}

/** Onboarding takes the screen first; the flag stays unwritten so the notes show on a later launch. */
export function shouldOpenNotes(input: { noteCount: number; onboardingPending: boolean; pathname: string }): boolean {
    if (input.noteCount === 0 || input.onboardingPending) return false;
    return !input.pathname.startsWith("/onboarding") && !input.pathname.startsWith(WHATS_NEW_ROUTE);
}
