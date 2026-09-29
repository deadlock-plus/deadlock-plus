<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { TriangleAlert } from "@lucide/svelte";

    import Badge from "$lib/components/ui/badge.svelte";
    import Button from "$lib/components/ui/button.svelte";
    import Input from "$lib/components/ui/input.svelte";
    import Flag from "$lib/components/flag.svelte";

    import PingCard from "$lib/features/connection/components/ping-card.svelte";
    import Sparkline from "$lib/features/connection/components/sparkline.svelte";
    import { networkHistory, networkSnapshot, startNetworkMonitor } from "$lib/features/connection/api";
    import { readExitLagOffset, writeExitLagOffset } from "$lib/features/connection/settings";
    import type { HistoryPoint, NetworkSnapshot } from "$lib/features/connection/types";

    const HISTORY_SHOWN = 300;

    let snap = $state<NetworkSnapshot | null>(null);
    let history = $state<HistoryPoint[]>([]);
    let offset = $state<number | null>(null);
    let entered = $state("");

    const relay = $derived(snap?.relay ?? null);
    const exitEndpoint = $derived(snap?.exitlagEndpoints.find((e) => e.isExit) ?? null);
    const appliedOffset = $derived(offset ?? 0);
    const rawAvg = $derived(relay?.ping.avg ?? null);
    const routedAvg = $derived(exitEndpoint?.ping.avg != null ? exitEndpoint.ping.avg + appliedOffset : null);
    const saved = $derived(rawAvg != null && routedAvg != null ? rawAvg - routedAvg : null);

    const shown = $derived(history.slice(-HISTORY_SHOWN));
    const series = $derived([
        { label: "Without ExitLag", color: "var(--muted-foreground)", values: shown.map((p) => p.raw) },
        {
            label: "With ExitLag (est.)",
            color: "var(--success)",
            values: shown.map((p) => (p.exit == null ? null : p.exit + appliedOffset)),
        },
    ]);

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
        const value = Number.parseFloat(entered);
        const measured = exitEndpoint?.ping.avg;
        if (!Number.isFinite(value) || measured == null) {
            toast.error("Enter the ping ExitLag shows while it's connected and sampling.");
            return;
        }
        offset = Math.round((value - measured) * 10) / 10;
        entered = "";
        await writeExitLagOffset(offset);
        toast.success(`Calibrated: exit server ping ${offset >= 0 ? "+" : ""}${offset} ms`);
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

<div class="mx-auto flex max-w-4xl flex-col gap-4 px-6 py-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">Connection</h1>
            <p class="text-sm text-muted-foreground">Live server, ping and packet loss, with and without ExitLag.</p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <Badge variant={snap?.gameRunning ? "success" : "outline"}
                >Deadlock {snap?.gameRunning ? "running" : "not running"}</Badge
            >
            <Badge variant={snap?.exitlagRunning ? "success" : "outline"}
                >ExitLag {snap?.exitlagRunning ? "running" : "not running"}</Badge
            >
        </div>
    </header>

    {#if snap?.needsPermission && !snap.traceError}
        <div class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2 text-sm">
            <span>
                Live monitoring needs to watch your network traffic while Deadlock runs. Your system will ask for your
                password.
            </span>
            <Button size="sm" onclick={retry}>Allow</Button>
        </div>
    {/if}

    {#if snap?.traceError}
        <div
            class="flex items-center justify-between gap-3 rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning"
        >
            <span class="flex items-center gap-2"><TriangleAlert class="size-4 shrink-0" />{snap.traceError}</span>
            <Button size="sm" variant="outline" onclick={retry}>Retry</Button>
        </div>
    {/if}

    <section class="rounded-lg border border-border bg-card p-4">
        <h2 class="mb-3 text-sm font-medium">Current server</h2>
        {#if !snap?.gameRunning}
            <p class="text-sm text-muted-foreground">Deadlock isn't running.</p>
        {:else if !relay}
            <p class="text-sm text-muted-foreground">Not connected to a match server yet. Start or join a match.</p>
        {:else}
            <div class="flex flex-wrap items-center gap-x-6 gap-y-2">
                <div class="flex items-center gap-3">
                    <Flag code={relay.countryCode} />
                    <div>
                        <div class="text-base font-medium">{relay.description ?? "Unknown relay"}</div>
                        <div class="font-mono text-xs text-muted-foreground">
                            {relay.popCode ? `${relay.popCode} · ` : ""}{relay.ip}:{relay.port}
                        </div>
                    </div>
                </div>
                <dl class="flex gap-6 text-xs">
                    <div>
                        <dt class="text-muted-foreground">in</dt>
                        <dd class="tabular-nums">{Math.round(relay.ppsIn)} pkt/s</dd>
                    </div>
                    <div>
                        <dt class="text-muted-foreground">out</dt>
                        <dd class="tabular-nums">{Math.round(relay.ppsOut)} pkt/s</dd>
                    </div>
                    <div>
                        <dt class="text-muted-foreground">longest inbound gap (1 s)</dt>
                        <dd>
                            <Badge
                                variant={relay.maxGapMs < 100
                                    ? "success"
                                    : relay.maxGapMs < 250
                                      ? "warning"
                                      : "destructive"}
                            >
                                {Math.round(relay.maxGapMs)} ms
                            </Badge>
                        </dd>
                    </div>
                </dl>
            </div>
        {/if}
    </section>

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
                : `Ping to ExitLag's exit server ${offset >= 0 ? "+" : ""}${offset} ms calibrated last hop.`}
            stats={exitEndpoint?.ping ?? null}
            offset={appliedOffset}
            estimate
            unavailable={!snap?.exitlagRunning
                ? "ExitLag isn't running."
                : exitEndpoint
                  ? undefined
                  : "Waiting for ExitLag to carry Deadlock traffic."}
        />
        <div class="flex flex-col gap-3 rounded-lg border border-border bg-card p-4">
            <span class="text-sm font-medium">ExitLag difference</span>
            {#if saved == null}
                <p class="py-6 text-sm text-muted-foreground">Needs both pings.</p>
            {:else}
                <div class="flex items-baseline gap-1">
                    <span
                        class="text-4xl font-semibold tabular-nums {saved >= 0 ? 'text-success' : 'text-destructive'}"
                    >
                        {saved >= 0 ? "−" : "+"}{Math.abs(Math.round(saved))}
                    </span>
                    <span class="text-sm text-muted-foreground">ms</span>
                </div>
                <p class="text-xs text-muted-foreground">
                    {saved >= 0
                        ? "ExitLag is faster than your direct route."
                        : "Your direct route is faster than ExitLag."}
                </p>
            {/if}
        </div>
    </div>

    <section class="rounded-lg border border-border bg-card p-4">
        <div class="mb-2 flex items-center justify-between">
            <h2 class="text-sm font-medium">Last {Math.round(shown.length / 60)} min</h2>
            <div class="flex gap-4 text-xs text-muted-foreground">
                {#each series as s (s.label)}
                    <span class="flex items-center gap-1.5"
                        ><span class="inline-block h-0.5 w-4" style="background:{s.color}"></span>{s.label}</span
                    >
                {/each}
            </div>
        </div>
        {#if shown.length < 2}
            <p class="py-8 text-center text-sm text-muted-foreground">Ping history appears once you're in a match.</p>
        {:else}
            <Sparkline {series} />
        {/if}
    </section>

    {#if snap && snap.exitlagEndpoints.length > 0}
        <section class="rounded-lg border border-border bg-card p-4">
            <h2 class="mb-2 text-sm font-medium">ExitLag path</h2>
            <div class="flex flex-col gap-1.5">
                {#each snap.exitlagEndpoints as e (e.ip + e.port)}
                    <div class="flex items-center gap-3 text-xs">
                        <Badge variant={e.isExit ? "success" : "secondary"}
                            >{e.isExit ? "exit server" : "entry point"}</Badge
                        >
                        <span class="font-mono">{e.ip}:{e.port}</span>
                        <span class="text-muted-foreground tabular-nums">{Math.round(e.pps)} pkt/s</span>
                        <span class="ml-auto tabular-nums"
                            >{e.ping.avg == null ? "—" : `${Math.round(e.ping.avg)} ms`}</span
                        >
                    </div>
                {/each}
            </div>
        </section>
    {/if}

    <section class="rounded-lg border border-border bg-card p-4">
        <h2 class="text-sm font-medium">Calibrate ExitLag estimate</h2>
        <p class="mt-1 text-xs text-muted-foreground">
            The estimate is the exit server's ping plus a fixed last hop to the game server. Enter the ping ExitLag
            shows right now and the offset is computed once and saved.
        </p>
        <div class="mt-3 flex items-center gap-2">
            <Input
                bind:value={entered}
                type="number"
                placeholder="ExitLag shows (ms)"
                aria-label="ExitLag latency in milliseconds"
                class="w-48"
            />
            <Button size="sm" onclick={calibrate} disabled={exitEndpoint?.ping.avg == null}>Calibrate</Button>
            {#if offset != null}
                <Button size="sm" variant="ghost" onclick={resetCalibration}
                    >Reset ({offset >= 0 ? "+" : ""}{offset} ms)</Button
                >
            {/if}
        </div>
    </section>
</div>
