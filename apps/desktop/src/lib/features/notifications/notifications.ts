import { t } from "$lib/core/i18n.svelte";
import type { AppNotification } from "$lib/generated/types/AppNotification";

export type { AppNotification };
export const unreadCount = (items: AppNotification[]): number => items.filter((n) => !n.read).length;

export function formatRelative(unixSeconds: number): string {
    const minutes = Math.max(0, Math.round((Date.now() - unixSeconds * 1000) / 60000));
    if (minutes < 1) return t("notification_list.relative.now");
    if (minutes < 60) return t("notification_list.relative.minutes", { count: minutes });
    const hours = Math.round(minutes / 60);
    if (hours < 24) return t("notification_list.relative.hours", { count: hours });
    const days = Math.round(hours / 24);
    return days === 1
        ? t("notification_list.relative.yesterday")
        : t("notification_list.relative.days", { count: days });
}
