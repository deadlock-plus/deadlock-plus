import type { CrashKind, CrashReport } from "./crash";

export type { CrashReport };

/** Routes that take the screen first: the dialog waits until the user has left them. */
const BLOCKING_ROUTES = ["/onboarding", "/whats-new"];

export function crashHeading(kind: CrashKind): string {
    return kind === "unclean-exit" ? "Deadlock+ may have crashed last time" : "Deadlock+ hit an error last time";
}

export function crashDetail(kind: CrashKind): string {
    return kind === "unclean-exit"
        ? "It did not shut down cleanly. That can follow a crash, a forced close or a power loss."
        : "A report was saved on this computer.";
}

export function crashTimeLabel(timestampMs: number): string {
    return new Date(timestampMs).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export function shouldShowCrash(input: {
    report: CrashReport | null;
    pathname: string;
    dismissed: boolean;
    closed: boolean;
}): boolean {
    if (input.report === null || input.dismissed || input.closed) return false;
    return !BLOCKING_ROUTES.some((route) => input.pathname === route || input.pathname.startsWith(`${route}/`));
}

export function reportFileName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
}

export function attachNote(path: string): string {
    return `Attach ${reportFileName(path)} to the issue. It is saved at ${path}.`;
}
