import { formatDate, t } from "$lib/core/i18n.svelte";
import type { CrashKind, CrashReport } from "./crash";

export type { CrashReport };

/** Routes that take the screen first: the dialog waits until the user has left them. */
const BLOCKING_ROUTES = ["/onboarding", "/whats-new"];

export function crashHeading(kind: CrashKind): string {
    return kind === "unclean-exit" ? t("crash.heading_unclean") : t("crash.heading_error");
}

export function crashDetail(kind: CrashKind): string {
    return kind === "unclean-exit" ? t("crash.detail_unclean") : t("crash.detail_error");
}

export function crashTimeLabel(timestampMs: number): string {
    return formatDate(timestampMs, { dateStyle: "medium", timeStyle: "short" });
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
    return t("crash.attach_note", { file: reportFileName(path), path });
}
