import { toast } from "svelte-sonner";
import { errorText } from "$lib/core/errors";
import { t } from "$lib/core/i18n.svelte";
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
    toast(t("support.nudge"), {
        duration: 20000,
        action: {
            label: t("support.action"),
            onClick: () =>
                void openUrl(SUPPORT_URL).catch((e) =>
                    toast.error(t("support.open_link_error", { error: errorText(e) })),
                ),
        },
    });
}
