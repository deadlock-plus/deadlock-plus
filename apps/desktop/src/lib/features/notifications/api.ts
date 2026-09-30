import { command, listen, type Unlisten } from "$lib/core/tauri";
import type { AppNotification } from "./notifications";

const CHANGED_EVENT = "notifications-changed";

export const listNotifications = () => command<AppNotification[]>("list_notifications");
export const markNotificationsRead = () => command("mark_notifications_read");
export const markNotificationRead = (id: string) => command("mark_notification_read", { id });
export const onNotificationsChanged = (handler: () => void): Promise<Unlisten> => listen(CHANGED_EVENT, handler);
