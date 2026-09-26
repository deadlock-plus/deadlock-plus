import { toast } from "svelte-sonner";
import { getAppInfo } from "$lib/features/settings/about";
import { settings } from "$lib/features/settings/settings.svelte";
import { settingsUi } from "$lib/features/settings/ui.svelte";
import { connectivity } from "$lib/features/connectivity/online.svelte";
import { updater } from "./updater.svelte";

/** Silent on failure and in dev builds; only an available update surfaces, as a toast. */
export async function checkOnLaunch() {
    await settings.ready;
    if (!settings.autoUpdateCheck || !connectivity.online) return;
    try {
        if ((await getAppInfo()).debugBuild) return;
    } catch {
        return;
    }
    if (!(await updater.check())) return;
    toast(`Deadlock+ ${updater.version} is available`, {
        duration: 15000,
        action: { label: "Details", onClick: () => settingsUi.show("about") },
    });
}
