import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { unreadCount, type Alert } from "./alerts";

const CHANGED_EVENT = "alerts-changed";

class AlertsStore {
    items = $state<Alert[]>([]);
    unread = $derived(unreadCount(this.items));

    async refresh() {
        try {
            this.items = await invoke<Alert[]>("list_alerts");
        } catch {
            // Not running inside Tauri.
        }
    }

    async fetchNow() {
        try {
            await invoke("refresh_alerts");
        } catch {
            // Not running inside Tauri.
        }
        await this.refresh();
    }

    async markAllRead() {
        if (this.unread === 0) return;
        try {
            await invoke("mark_alerts_read");
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

export const alerts = new AlertsStore();
