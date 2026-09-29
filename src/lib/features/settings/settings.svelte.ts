import { invoke } from "@tauri-apps/api/core";
import { kvGet, kvSet } from "$lib/kv";
import { resolveIngestConsent } from "./ingest-consent";
import { DEFAULT_SCHEDULE, type MaintenanceSchedule } from "./maintenance";
import { DEFAULT_THEME, resolveMotionPreference, resolveTheme, type MotionPreference, type ThemeId } from "./themes";

const STORE = "app-settings";
const ACCESSIBLE_FONT_KEY = "accessibleFont";
const INGEST_KEY = "matchIngest";
const CLOSE_TO_TRAY_KEY = "closeToTray";
const MAINTENANCE_KEY = "maintenanceSchedule";
const ALERTS_KEY = "updateAlerts";
const BREAK_HINT_KEY = "breakHint";
const THEME_KEY = "theme";
const MOTION_KEY = "motion";
const AUTO_UPDATE_KEY = "autoUpdateCheck";
const AUTO_SCAN_ADDONS_KEY = "autoScanAddons";

// Each key is read on its own so one unreadable value keeps its default without discarding the rest.
async function stored<T>(key: string): Promise<T | undefined> {
    try {
        return (await kvGet<T>(STORE, key)) ?? undefined;
    } catch {
        return undefined;
    }
}

class Settings {
    accessibleFont = $state(false);
    theme = $state<ThemeId>(DEFAULT_THEME);
    motion = $state<MotionPreference>("system");
    matchIngest = $state(false);
    ingestPromptPending = $state(false);
    closeToTray = $state(false);
    maintenance = $state<MaintenanceSchedule>({ ...DEFAULT_SCHEDULE });
    updateAlerts = $state(false);
    breakHint = $state(false);
    autoUpdateCheck = $state(true);
    autoScanAddons = $state(true);

    private resolveReady: () => void = () => {};
    ready = new Promise<void>((resolve) => (this.resolveReady = resolve));

    async init() {
        this.accessibleFont = (await stored<boolean>(ACCESSIBLE_FONT_KEY)) ?? false;
        try {
            const consent = resolveIngestConsent(await kvGet<boolean>(STORE, INGEST_KEY));
            this.matchIngest = consent.enabled;
            this.ingestPromptPending = consent.needsPrompt;
        } catch {
            // An unreadable answer must not re-ask the question: stay off and silent.
        }
        this.closeToTray = (await stored<boolean>(CLOSE_TO_TRAY_KEY)) ?? false;
        const schedule = await stored<Partial<MaintenanceSchedule>>(MAINTENANCE_KEY);
        this.maintenance = { ...DEFAULT_SCHEDULE, ...schedule };
        this.updateAlerts = (await stored<boolean>(ALERTS_KEY)) ?? false;
        this.breakHint = (await stored<boolean>(BREAK_HINT_KEY)) ?? false;
        this.theme = resolveTheme(await stored<string>(THEME_KEY));
        this.motion = resolveMotionPreference(await stored<string>(MOTION_KEY));
        this.autoUpdateCheck = (await stored<boolean>(AUTO_UPDATE_KEY)) ?? true;
        this.autoScanAddons = (await stored<boolean>(AUTO_SCAN_ADDONS_KEY)) ?? true;
        await this.applyIngest();
        await this.applyCloseToTray();
        await this.applyMaintenance();
        await this.applyUpdateAlerts();
        this.resolveReady();
    }

    private async applyUpdateAlerts() {
        try {
            await invoke("set_alerts_enabled", { enabled: this.updateAlerts });
        } catch {
            // Not running inside Tauri.
        }
    }

    async setUpdateAlerts(value: boolean) {
        this.updateAlerts = value;
        await this.applyUpdateAlerts();
        try {
            await kvSet(STORE, ALERTS_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setTheme(value: ThemeId) {
        this.theme = value;
        try {
            await kvSet(STORE, THEME_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setMotion(value: MotionPreference) {
        this.motion = value;
        try {
            await kvSet(STORE, MOTION_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setAutoUpdateCheck(value: boolean) {
        this.autoUpdateCheck = value;
        try {
            await kvSet(STORE, AUTO_UPDATE_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setAutoScanAddons(value: boolean) {
        this.autoScanAddons = value;
        try {
            await kvSet(STORE, AUTO_SCAN_ADDONS_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setBreakHint(value: boolean) {
        this.breakHint = value;
        try {
            await kvSet(STORE, BREAK_HINT_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    private async applyMaintenance() {
        try {
            await invoke("set_maintenance_schedule", { schedule: $state.snapshot(this.maintenance) });
        } catch {
            // Not running inside Tauri.
        }
    }

    async setMaintenance(patch: Partial<MaintenanceSchedule>) {
        this.maintenance = { ...this.maintenance, ...patch };
        await this.applyMaintenance();
        try {
            await kvSet(STORE, MAINTENANCE_KEY, $state.snapshot(this.maintenance));
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    private async applyCloseToTray() {
        try {
            await invoke("set_close_to_tray", { enabled: this.closeToTray });
        } catch {
            // Not running inside Tauri.
        }
    }

    async setCloseToTray(value: boolean) {
        this.closeToTray = value;
        await this.applyCloseToTray();
        try {
            await kvSet(STORE, CLOSE_TO_TRAY_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    private async applyIngest() {
        try {
            await invoke("set_ingest_enabled", { enabled: this.matchIngest });
        } catch {
            // Not running inside Tauri.
        }
    }

    async setMatchIngest(value: boolean) {
        this.matchIngest = value;
        this.ingestPromptPending = false;
        await this.applyIngest();
        try {
            await kvSet(STORE, INGEST_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    async setAccessibleFont(value: boolean) {
        this.accessibleFont = value;
        try {
            await kvSet(STORE, ACCESSIBLE_FONT_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }
}

export const settings = new Settings();
