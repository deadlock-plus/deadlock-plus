<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";

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
    import { networkHistory, networkSnapshot, startNetworkMonitor } from "$lib/features/connection/api";
    import {
        calibratedOffset,
        exitLagSaved,
        formatOffset,
        historySeries,
        routedAverage,
    } from "$lib/features/connection/connection";
    import { readExitLagOffset, writeExitLagOffset } from "$lib/features/connection/settings";
    import type { HistoryPoint, NetworkSnapshot } from "$lib/features/connection/types";

    let snap = $state<NetworkSnapshot | null>(null);
    let history = $state<HistoryPoint[]>([]);
    let offset = $state<number | null>(null);
    let entered = $state("");

    const relay = $derived(snap?.relay ?? null);
    const exitEndpoint = $derived(snap?.exitlagEndpoints.find((e) => e.isExit) ?? null);
    const appliedOffset = $derived(offset ?? 0);
    const saved = $derived(exitLagSaved(relay?.ping.avg ?? null, routedAverage(exitEndpoint?.ping.avg, appliedOffset)));
    const chart = $derived(historySeries(history, appliedOffset));

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
            toast.error("Enter the ping ExitLag shows while it's connected and sampling.");
            return;
        }
        offset = next;
        entered = "";
        await writeExitLagOffset(offset);
        toast.success(`Calibrated: exit server ping ${formatOffset(offset)} ms`);
    }

    async function resetCalibration() {
        offset = null;
        await writeExitLagOffset(null);
    }

    onMount(() => {
        void readExitLagOffset().then((v) => (offset = v));
        void startNetworkMonitor().then(refresh);
        const timer = setInterval(refresh, 1000);
        return () => clearInterval(timer);
    });
</script>

<Page>
    <PageHeader title="Connection" subtitle="Live server, ping and packet loss, with and without ExitLag.">
        {#snippet actions()}
            <Badge variant={snap?.gameRunning ? "success" : "outline"}
                >Deadlock {snap?.gameRunning ? "running" : "not running"}</Badge
            >
            <Badge variant={snap?.exitlagRunning ? "success" : "outline"}
                >ExitLag {snap?.exitlagRunning ? "running" : "not running"}</Badge
            >
        {/snippet}
    </PageHeader>

    <MonitorNotices {snap} onretry={retry} />

    <CurrentServer gameRunning={snap?.gameRunning ?? false} {relay} />

    <div class="grid gap-4 md:grid-cols-3">
        <PingCard
            title="Without ExitLag"
            note="Direct ICMP ping to the relay you're connected to, over your normal route."
            stats={relay?.ping ?? null}
            unavailable={relay ? undefined : "Waiting for a match server."}
        />
        <PingCard
            title="With ExitLag"
            note={offset == null
                ? "Ping to ExitLag's exit server. Calibrate below to add the last hop and match what ExitLag shows."
                : `Ping to ExitLag's exit server ${formatOffset(offset)} ms calibrated last hop.`}
            stats={exitEndpoint?.ping ?? null}
            offset={appliedOffset}
            estimate
            unavailable={!snap?.exitlagRunning
                ? "ExitLag isn't running."
                : exitEndpoint
                  ? undefined
                  : "Waiting for ExitLag to carry Deadlock traffic."}
        />
        <DifferenceCard {saved} />
    </div>

    <HistoryCard shown={chart.shown} series={chart.series} />

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
</Page>
