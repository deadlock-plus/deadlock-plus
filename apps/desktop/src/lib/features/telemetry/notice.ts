import { toast } from "svelte-sonner";
import { goto } from "$app/navigation";
import { kvGet } from "$lib/core/kv";
import { t } from "$lib/core/i18n.svelte";
import { needsOnboarding } from "$lib/features/onboarding/onboarding";
import { settings } from "$lib/features/settings/settings.svelte";
import { needsLaunchNotice } from "./consent";

let started = false;

/** Tells users who finished onboarding before telemetry existed, once. Nothing is sent until they have seen it. */
export async function noticeOnLaunch() {
    if (started) return;
    started = true;
    await settings.ready;
    let onboardingPending: boolean;
    try {
        onboardingPending = needsOnboarding(await kvGet<number>("app-settings", "onboardingVersion"));
    } catch {
        return;
    }
    const choice = { enabled: settings.telemetry, noticeShown: settings.telemetryNoticeShown };
    if (!needsLaunchNotice(choice, onboardingPending)) return;
    toast(t("telemetry.notice"), {
        duration: 20000,
        action: { label: t("telemetry.notice_action"), onClick: () => void goto("/settings/privacy") },
    });
    await settings.markTelemetryNoticeShown();
}
