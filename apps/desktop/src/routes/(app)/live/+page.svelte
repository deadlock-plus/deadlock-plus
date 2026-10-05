<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";

    import { t } from "$lib/core/i18n.svelte";
    import { createPoller } from "$lib/core/poller";
    import { platform } from "$lib/core/platform";
    import Badge from "$lib/ui/badge.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";

    import Calibration from "$lib/features/connection/components/calibration.svelte";
    import CurrentServer from "$lib/features/connection/components/current-server.svelte";
    import DifferenceCard from "$lib/features/connection/components/difference-card.svelte";
    import ExitLagPath from "$lib/features/connection/components/exitlag-path.svelte";
    import HistoryCard from "$lib/features/connection/components/history-card.svelte";
    import MonitorNotices from "$lib/features/connection/components/monitor-notices.svelte";
    import PingCard from "$lib/features/connection/components/ping-card.svelte";
    import { live } from "$lib/features/live/live.svelte";
    import { stateLine } from "$lib/features/live/live";
    import { networkHistory, networkSnapshot, startNetworkMonitor } from "$lib/features/connection/api";
    import {
        calibratedOffset,
        connectionSubtitle,
        exitLagAvailable,
        exitLagSaved,
        formatOffset,
        historySeries,
        routedAverage,
    } from "$lib/features/connection/connection";
    import { readExitLagOffset, writeExitLagOffset } from "$lib/features/connection/settings";
    import type { HistoryPoint, NetworkSnapshot } from "$lib/features/connection/types";

    const REFRESH_MS = 1000;
    const exitLag = exitLagAvailable(platform);

    let snap = $state<NetworkSnapshot | null>(null);
    let history = $state<HistoryPoint[]>([]);
    let offset = $state<number | null>(null);
    let entered = $state("");

    const relay = $derived(snap?.relay ?? null);
    const exitEndpoint = $derived(snap?.exitlagEndpoints.find((e) => e.isExit) ?? null);
    const appliedOffset = $derived(offset ?? 0);
    const saved = $derived(exitLagSaved(relay?.ping.avg ?? null, routedAverage(exitEndpoint?.ping.avg, appliedOffset)));
    const chart = $derived(historySeries(history, appliedOffset, exitLag));
    const line = $derived(live.state ? stateLine(live.state.phase) : null);

    async function refresh() {
        try {
            [snap, history] = await Promise.all([networkSnapshot(), networkHistory()]);
        } catch {
            // Transient invoke failures just skip a tick.
        }
    }

    async function retry() {
        await startNetworkMonitor(true);
        await refresh();
    }

    async function calibrate() {
        const next = calibratedOffset(entered, exitEndpoint?.ping.avg);
        if (next === null) {
            toast.error(t("connection.page.calibrate_invalid"));
            return;
        }
        offset = next;
        entered = "";
        await writeExitLagOffset(offset);
        toast.success(t("connection.page.calibrated", { offset: formatOffset(offset) }));
    }

    async function resetCalibration() {
        offset = null;
        await writeExitLagOffset(null);
    }

    onMount(() => {
        void readExitLagOffset().then((v) => (offset = v));
        void startNetworkMonitor().then(refresh);
        const stopLive = live.start();
        const stopPolling = createPoller(refresh, { intervalMs: REFRESH_MS }).start();
        return () => {
            stopLive();
            stopPolling();
        };
    });
</script>

<Page>
    <PageHeader title={t("connection.page.title")} subtitle={connectionSubtitle(platform)}>
        {#snippet actions()}
            <Badge variant={snap?.gameRunning ? "success" : "outline"}>
                {snap?.gameRunning ? t("connection.page.deadlock_running") : t("connection.page.deadlock_not_running")}
            </Badge>
            {#if exitLag}
                <Badge variant={snap?.exitlagRunning ? "success" : "outline"}>
                    {snap?.exitlagRunning
                        ? t("connection.page.exitlag_running")
                        : t("connection.page.exitlag_not_running")}
                </Badge>
            {/if}
        {/snippet}
    </PageHeader>

    {#if line}
        <p class="text-sm text-muted-foreground">
            {t(line.key)}
        </p>
    {/if}

    {#if live.state?.matchPresent}
        <section>
            <h2 class="text-base font-semibold">{t("live.match.title")}</h2>
        </section>
    {/if}

    <MonitorNotices {snap} onretry={retry} />

    <CurrentServer gameRunning={snap?.gameRunning ?? false} {relay} />

    <div class={["grid gap-4", exitLag && "md:grid-cols-3"]}>
        <PingCard
            title={exitLag ? t("connection.series.without") : t("connection.series.ping")}
            note={t("connection.page.direct_note")}
            stats={relay?.ping ?? null}
            unavailable={relay ? undefined : t("connection.page.waiting_server")}
        />
        {#if exitLag}
            <PingCard
                title={t("connection.page.with_exitlag")}
                note={offset == null
                    ? t("connection.page.exit_note")
                    : t("connection.page.exit_note_calibrated", { offset: formatOffset(offset) })}
                stats={exitEndpoint?.ping ?? null}
                offset={appliedOffset}
                estimate
                unavailable={!snap?.exitlagRunning
                    ? t("connection.page.exitlag_idle")
                    : exitEndpoint
                      ? undefined
                      : t("connection.page.waiting_exitlag")}
            />
            <DifferenceCard {saved} />
        {/if}
    </div>

    <HistoryCard shown={chart.shown} series={chart.series} />

    {#if exitLag}
        {#if snap && snap.exitlagEndpoints.length > 0}
            <ExitLagPath endpoints={snap.exitlagEndpoints} />
        {/if}

        <Calibration
            bind:entered
            {offset}
            canCalibrate={exitEndpoint?.ping.avg != null}
            oncalibrate={calibrate}
            onreset={resetCalibration}
        />
    {/if}
</Page>
