<script lang="ts">
    import "../app.css";
    import { command } from "$lib/core/tauri";
    import { prefs } from "$lib/core/prefs";
    import { onMount } from "svelte";
    import { Toaster } from "svelte-sonner";
    import Sidebar from "$lib/components/sidebar.svelte";
    import Statusbar from "$lib/components/statusbar.svelte";
    import Titlebar from "$lib/components/titlebar.svelte";
    import OfflineBanner from "$lib/features/connectivity/components/offline-banner.svelte";
    import { connectivity } from "$lib/features/connectivity/online.svelte";
    import ErrorPanel from "$lib/features/errors/components/error-panel.svelte";
    import { installFrontendLogging } from "$lib/features/logging/frontend";
    import { apiHealth } from "$lib/features/api-health/health.svelte";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { alerts } from "$lib/features/alerts/alerts.svelte";
    import { notifications } from "$lib/features/notifications/notifications.svelte";
    import { jobs } from "$lib/features/jobs/jobs.svelte";
    import { performanceScan } from "$lib/features/performance/scan.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import IngestPrompt from "$lib/features/settings/components/ingest-prompt.svelte";
    import OnboardingDialog from "$lib/features/onboarding/components/onboarding-dialog.svelte";
    import { onboarding } from "$lib/features/onboarding/onboarding.svelte";
    import WhatsNewDialog from "$lib/features/updates/components/whats-new-dialog.svelte";
    import { checkOnLaunch, startBackgroundUpdateChecks } from "$lib/features/updates/launch-check";
    import { updater } from "$lib/features/updates/updater.svelte";
    import { whatsNew } from "$lib/features/updates/whats-new.svelte";
    import SettingsOverlay from "$lib/features/settings/components/settings-overlay.svelte";
    import { isLightTheme, resolveReducedMotion } from "$lib/features/settings/themes";

    let { children } = $props();
    let collapsed = $state(false);

    let osReducedMotion = $state(false);

    $effect(() => {
        document.documentElement.toggleAttribute("data-accessible-font", settings.accessibleFont);
    });

    $effect(() => {
        document.documentElement.dataset.theme = settings.theme;
    });

    $effect(() => {
        document.documentElement.toggleAttribute(
            "data-reduced-motion",
            resolveReducedMotion(settings.motion, osReducedMotion),
        );
    });

    $effect(() => {
        const available = updater.phase === "available" || updater.phase === "downloading";
        command("set_update_badge", { available }).catch(() => {});
    });

    onMount(() => {
        const stopLogging = installFrontendLogging();
        settings.init();
        const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
        osReducedMotion = motionQuery.matches;
        const onMotionChange = (e: MediaQueryListEvent) => (osReducedMotion = e.matches);
        motionQuery.addEventListener("change", onMotionChange);
        const stopPolling = ingestStatus.start();
        const stopApiHealth = apiHealth.start();
        const stopAccount = steamAccount.start();
        const stopAlerts = alerts.start();
        const stopNotifications = notifications.start();
        const stopJobs = jobs.start();
        const stopPerformanceScan = performanceScan.start();
        const stopConnectivity = connectivity.start();
        onboarding.init();
        whatsNew.init();
        checkOnLaunch();
        const stopUpdateChecks = startBackgroundUpdateChecks();
        collapsed = prefs.getBool("sidebarCollapsed", false);
        // Two rAFs: the first fires before the browser has painted this frame, the second
        // guarantees one already happened. The window is built hidden so it's only ever revealed
        // with a real frame already rendered behind it, not a flash of empty/background-colored space.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                command("frontend_ready").catch((e) => console.error("frontend_ready failed:", e));
            });
        });
        return () => {
            motionQuery.removeEventListener("change", onMotionChange);
            stopLogging();
            stopPolling();
            stopApiHealth();
            stopAccount();
            stopAlerts();
            stopNotifications();
            stopJobs();
            stopPerformanceScan();
            stopConnectivity();
            stopUpdateChecks();
        };
    });

    function toggleSidebar() {
        collapsed = !collapsed;
        prefs.setBool("sidebarCollapsed", collapsed);
    }
</script>

<div class="flex h-screen w-screen flex-col overflow-hidden bg-chrome text-foreground">
    <Titlebar sidebarCollapsed={collapsed} onToggleSidebar={toggleSidebar} />

    <div class="flex min-h-0 flex-1">
        <Sidebar {collapsed} />

        <main class="noir-panel min-w-0 flex-1 overflow-y-auto rounded-l-xl border-y border-l border-border">
            <OfflineBanner />
            {@render children?.()}
        </main>
    </div>

    <Statusbar />
</div>

<SettingsOverlay {collapsed} />
<OnboardingDialog />
<IngestPrompt />
<WhatsNewDialog />

<Toaster richColors position="bottom-right" theme={isLightTheme(settings.theme) ? "light" : "dark"} />
