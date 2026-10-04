import {
    setAlertsEnabled,
    setCloseToTray,
    setGcRecoveryEnabled,
    setIngestEnabled,
    setMaintenanceSchedule,
    setPostgameCaptureEnabled,
} from "./api";
import { kvGet, kvSet } from "$lib/core/kv";
import { setPresenceHeroNames, setPresenceSettings } from "$lib/features/presence/api";
import { loadHeroes } from "$lib/features/heroes/heroes";
import { DEFAULT_PRESENCE, heroNameMap, resolvePresence } from "$lib/features/presence/presence";
import type { PresenceSettings } from "$lib/generated/types/PresenceSettings";
import { resolveIngestConsent } from "./ingest-consent";
import { DEFAULT_SCHEDULE, type MaintenanceSchedule } from "./maintenance";
import { DEFAULT_THEME, resolveMotionPreference, resolveTheme, type MotionPreference, type ThemeId } from "./themes";

const STORE = "app-settings";
const ACCESSIBLE_FONT_KEY = "accessibleFont";
const INGEST_KEY = "matchIngest";
const GC_RECOVERY_KEY = "gcRecovery";
const POSTGAME_CAPTURE_KEY = "postgameCapture";
const CLOSE_TO_TRAY_KEY = "closeToTray";
const MAINTENANCE_KEY = "maintenanceSchedule";
const ALERTS_KEY = "updateAlerts";
const BREAK_HINT_KEY = "breakHint";
const THEME_KEY = "theme";
const MOTION_KEY = "motion";
const AUTO_UPDATE_KEY = "autoUpdateCheck";
const PRESENCE_KEY = "discordPresence";

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
    gcRecovery = $state(false);
    postgameCapture = $state(false);
    closeToTray = $state(false);
    maintenance = $state<MaintenanceSchedule>({ ...DEFAULT_SCHEDULE });
    updateAlerts = $state(false);
    breakHint = $state(false);
    autoUpdateCheck = $state(true);
    presence = $state<PresenceSettings>(resolvePresence(DEFAULT_PRESENCE));

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
        this.gcRecovery = (await stored<boolean>(GC_RECOVERY_KEY)) ?? false;
        this.postgameCapture = (await stored<boolean>(POSTGAME_CAPTURE_KEY)) ?? false;
        this.closeToTray = (await stored<boolean>(CLOSE_TO_TRAY_KEY)) ?? false;
        const schedule = await stored<Partial<MaintenanceSchedule>>(MAINTENANCE_KEY);
        this.maintenance = { ...DEFAULT_SCHEDULE, ...schedule };
        this.updateAlerts = (await stored<boolean>(ALERTS_KEY)) ?? false;
        this.breakHint = (await stored<boolean>(BREAK_HINT_KEY)) ?? false;
        this.theme = resolveTheme(await stored<string>(THEME_KEY));
        this.motion = resolveMotionPreference(await stored<string>(MOTION_KEY));
        this.autoUpdateCheck = (await stored<boolean>(AUTO_UPDATE_KEY)) ?? true;
        this.presence = resolvePresence(await stored<unknown>(PRESENCE_KEY));
        await this.applyIngest();
        await this.applyGcRecovery();
        await this.applyPostgameCapture();
        await this.applyCloseToTray();
        await this.applyMaintenance();
        await this.applyUpdateAlerts();
        await this.applyPresence();
        this.resolveReady();
    }

    private async applyPresence() {
        try {
            await setPresenceSettings($state.snapshot(this.presence));
            if (this.presence.level === "detailed") await setPresenceHeroNames(heroNameMap(await loadHeroes()));
        } catch {
            // Not running inside Tauri.
        }
    }

    async setPresence(patch: Partial<PresenceSettings>) {
        this.presence = { ...this.presence, ...patch };
        await this.applyPresence();
        try {
            await kvSet(STORE, PRESENCE_KEY, $state.snapshot(this.presence));
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    private async applyUpdateAlerts() {
        try {
            await setAlertsEnabled(this.updateAlerts);
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
            await setMaintenanceSchedule($state.snapshot(this.maintenance));
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
            await setCloseToTray(this.closeToTray);
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
            await setIngestEnabled(this.matchIngest);
        } catch {
            // Not running inside Tauri.
        }
    }

    private async applyGcRecovery() {
        try {
            await setGcRecoveryEnabled(this.gcRecovery);
        } catch {
            // Not running inside Tauri.
        }
    }

    async setGcRecovery(value: boolean) {
        this.gcRecovery = value;
        await this.applyGcRecovery();
        try {
            await kvSet(STORE, GC_RECOVERY_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
        }
    }

    private async applyPostgameCapture() {
        try {
            await setPostgameCaptureEnabled(this.postgameCapture);
        } catch {
            // Not running inside Tauri.
        }
    }

    async setPostgameCapture(value: boolean) {
        this.postgameCapture = value;
        await this.applyPostgameCapture();
        try {
            await kvSet(STORE, POSTGAME_CAPTURE_KEY, value);
        } catch {
            // The choice just won't persist across restarts.
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
