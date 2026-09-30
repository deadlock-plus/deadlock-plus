import { listAlerts, markAlertsRead, onAlertsChanged, refreshAlerts } from "./api";
import { unreadCount, type Alert } from "./alerts";

class AlertsStore {
    items = $state<Alert[]>([]);
    unread = $derived(unreadCount(this.items));

    async refresh() {
        try {
            this.items = await listAlerts();
        } catch {
            // Not running inside Tauri.
        }
    }

    async fetchNow() {
        try {
            await refreshAlerts();
        } catch {
            // Not running inside Tauri.
        }
        await this.refresh();
    }

    async markAllRead() {
        if (this.unread === 0) return;
        try {
            await markAlertsRead();
        } catch {
            // Not running inside Tauri.
        }
    }

    start() {
        void this.refresh();
        const unlisten = onAlertsChanged(() => void this.refresh()).catch(() => () => {});
        return () => void unlisten.then((fn) => fn());
    }
}

export const alerts = new AlertsStore();
