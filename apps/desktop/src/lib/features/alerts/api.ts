import { command, listen, type Unlisten } from "$lib/core/tauri";
import type { Alert } from "./alerts";

const CHANGED_EVENT = "alerts-changed";

export const listAlerts = () => command<Alert[]>("list_alerts");
export const refreshAlerts = () => command("refresh_alerts");
export const markAlertsRead = () => command("mark_alerts_read");
export const onAlertsChanged = (handler: () => void): Promise<Unlisten> => listen(CHANGED_EVENT, handler);
