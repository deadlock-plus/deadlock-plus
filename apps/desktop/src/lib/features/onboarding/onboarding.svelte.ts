import { kvGet, kvSet } from "$lib/core/kv";
import { getAppInfo } from "$lib/features/settings/about";
import { settings } from "$lib/features/settings/settings.svelte";
import { gameStatus, needsOnboarding, type GameStatus } from "./onboarding";

const STORE = "app-settings";
const DONE_KEY = "onboardingDone";

class Onboarding {
    open = $state(false);
    game = $state<GameStatus>({ found: false, path: null });
    elevated = $state(false);

    async init() {
        await settings.ready;
        try {
            if (!needsOnboarding(await kvGet<boolean>(STORE, DONE_KEY))) return;
        } catch {
            // An unreadable flag must not trap the user in a dialog on every launch.
            return;
        }
        try {
            const info = await getAppInfo();
            this.game = gameStatus(info.gameDir);
            this.elevated = info.elevated;
        } catch {
            // Not running inside Tauri: show the steps without the game lookup.
        }
        this.open = true;
    }

    async finish() {
        this.open = false;
        try {
            await kvSet(STORE, DONE_KEY, true);
        } catch {
            // It will just show again next launch.
        }
    }
}

export const onboarding = new Onboarding();
