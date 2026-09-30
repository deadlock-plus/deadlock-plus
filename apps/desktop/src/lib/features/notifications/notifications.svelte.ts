import { listNotifications, markNotificationRead, markNotificationsRead, onNotificationsChanged } from "./api";
import { unreadCount, type AppNotification } from "./notifications";

class NotificationsStore {
    items = $state<AppNotification[]>([]);
    unread = $derived(unreadCount(this.items));

    async refresh() {
        try {
            this.items = await listNotifications();
        } catch {
            // Not running inside Tauri.
        }
    }

    async markAllRead() {
        if (this.unread === 0) return;
        try {
            await markNotificationsRead();
        } catch {
            // Not running inside Tauri.
        }
    }

    async markRead(id: string) {
        const item = this.items.find((n) => n.id === id);
        if (!item || item.read) return;
        try {
            await markNotificationRead(id);
        } catch {
            // Not running inside Tauri.
        }
    }

    start() {
        void this.refresh();
        const unlisten = onNotificationsChanged(() => void this.refresh()).catch(() => () => {});
        return () => void unlisten.then((fn) => fn());
    }
}

export const notifications = new NotificationsStore();
