import { toast } from "svelte-sonner";
import { openUrl } from "$lib/core/opener";
import { prefs } from "$lib/core/prefs";
import { SUPPORT_URL } from "./support";

const NUDGE_LAUNCH = 3;

let counted = false;

/** Counts this launch and shows a one-time support toast on the third. */
export function nudgeOnLaunch() {
    if (counted) return;
    counted = true;
    const launches = Number(prefs.getString("launchCount", "0")) + 1;
    prefs.setString("launchCount", String(launches));
    if (launches !== NUDGE_LAUNCH) return;
    toast("Enjoying Deadlock+? It is free and made by a small team.", {
        duration: 20000,
        action: {
            label: "Support Deadlock+",
            onClick: () => void openUrl(SUPPORT_URL).catch((e) => toast.error(`Could not open the link: ${e}`)),
        },
    });
}
