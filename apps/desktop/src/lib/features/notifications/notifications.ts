import type { AppNotification } from "$lib/generated/types/AppNotification";

export type { AppNotification };
export const unreadCount = (items: AppNotification[]): number => items.filter((n) => !n.read).length;

export function formatRelative(unixSeconds: number): string {
    const minutes = Math.max(0, Math.round((Date.now() - unixSeconds * 1000) / 60000));
    if (minutes < 1) return "just now";
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.round(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.round(hours / 24);
    return days === 1 ? "yesterday" : `${days}d ago`;
}
