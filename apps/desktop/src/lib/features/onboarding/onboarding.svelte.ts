import { goto } from "$app/navigation";
import { kvGet, kvSet } from "$lib/core/kv";
import { getAppInfo } from "$lib/features/settings/about";
import { settings } from "$lib/features/settings/settings.svelte";
import { finishOnboarding, gameStatus, isReturningUser, needsOnboarding, type GameStatus } from "./onboarding";

const STORE = "app-settings";
const VERSION_KEY = "onboardingVersion";
const LEGACY_KEY = "onboardingDone";
const ROUTE = "/onboarding";

class Onboarding {
    game = $state<GameStatus>({ found: false, path: null });
    elevated = $state(false);
    returning = $state(false);

    async init() {
        await settings.ready;
        try {
            if (!needsOnboarding(await kvGet<number>(STORE, VERSION_KEY))) return;
        } catch {
            // An unreadable flag must not trap the user in the setup on every launch.
            return;
        }
        await this.load();
        if (!location.pathname.startsWith(ROUTE)) await goto(ROUTE, { replaceState: true });
    }

    async load() {
        try {
            this.returning = isReturningUser(await kvGet<boolean>(STORE, LEGACY_KEY));
        } catch {
            // Treated as a new user.
        }
        try {
            const info = await getAppInfo();
            this.game = gameStatus(info.gameDir);
            this.elevated = info.elevated;
        } catch {
            // Not running inside Tauri: show the steps without the game lookup.
        }
    }

    async finish() {
        await finishOnboarding((version) => kvSet(STORE, VERSION_KEY, version));
        await goto("/");
    }
}

export const onboarding = new Onboarding();
