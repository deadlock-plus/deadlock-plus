import { toast } from "svelte-sonner";
import { getAppInfo } from "$lib/features/settings/about";
import { settings } from "$lib/features/settings/settings.svelte";
import { settingsUi } from "$lib/features/settings/ui.svelte";
import { connectivity } from "$lib/features/connectivity/online.svelte";
import { updater } from "./updater.svelte";

/** A release check is one cheap GitHub request; hourly is nowhere near enough traffic to matter. */
const RECHECK_MS = 60 * 60 * 1000;

/** Avoids re-toasting the same version on every periodic recheck while the app stays open. */
let notifiedVersion: string | null = null;

async function attemptCheck() {
    await settings.ready;
    if (!settings.autoUpdateCheck || !connectivity.online) return;
    try {
        if ((await getAppInfo()).debugBuild) return;
    } catch {
        return;
    }
    if (!(await updater.check())) return;
    if (updater.version === notifiedVersion) return;
    notifiedVersion = updater.version;
    toast(`Deadlock+ ${updater.version} is available`, {
        duration: 15000,
        action: { label: "Details", onClick: () => settingsUi.show("about") },
    });
}

/** Silent on failure and in dev builds; only an available update surfaces, as a toast. */
export async function checkOnLaunch() {
    await attemptCheck();
}

/** Rechecks on an interval for the rest of the session, since the app is often left open and actively used for hours. */
export function startBackgroundUpdateChecks() {
    const timer = setInterval(() => void attemptCheck(), RECHECK_MS);
    return () => clearInterval(timer);
}
