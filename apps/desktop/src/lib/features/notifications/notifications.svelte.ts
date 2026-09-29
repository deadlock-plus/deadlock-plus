import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { unreadCount, type AppNotification } from "./notifications";

const CHANGED_EVENT = "notifications-changed";

class NotificationsStore {
    items = $state<AppNotification[]>([]);
    unread = $derived(unreadCount(this.items));

    async refresh() {
        try {
            this.items = await invoke<AppNotification[]>("list_notifications");
        } catch {
            // Not running inside Tauri.
        }
    }

    async markAllRead() {
        if (this.unread === 0) return;
        try {
            await invoke("mark_notifications_read");
        } catch {
            // Not running inside Tauri.
        }
    }

    async markRead(id: string) {
        const item = this.items.find((n) => n.id === id);
        if (!item || item.read) return;
        try {
            await invoke("mark_notification_read", { id });
        } catch {
            // Not running inside Tauri.
        }
    }

    start() {
        void this.refresh();
        const unlisten = listen(CHANGED_EVENT, () => void this.refresh()).catch(() => () => {});
        return () => void unlisten.then((fn) => fn());
    }
}

export const notifications = new NotificationsStore();
