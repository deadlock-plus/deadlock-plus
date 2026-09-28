import type { Alert } from "$lib/generated/types/Alert";

export type { Alert };
export const unreadCount = (alerts: Alert[]): number => alerts.filter((a) => !a.read).length;

/** Routine posts stay quiet; anything else (matchmaking, major content) is called out. */
export function kindTone(kind: string): "routine" | "notable" {
    const k = kind.trim().toLowerCase();
    return k === "" || k === "patch notes" || k.startsWith("minor") ? "routine" : "notable";
}

/** Display label for a post's origin feed ("forum" or "steam", as reported by the backend). */
export function sourceLabel(source: string): "Steam" | "Forum" {
    return source === "steam" ? "Steam" : "Forum";
}

export function formatPublished(iso: string): string {
    const t = Date.parse(iso);
    if (Number.isNaN(t)) return "";
    return new Date(t).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

/** Feed links come from a third party, so only plain web links are opened. */
export function safeExternalUrl(link: string): string | null {
    try {
        const u = new URL(link);
        return u.protocol === "https:" || u.protocol === "http:" ? u.href : null;
    } catch {
        return null;
    }
}
