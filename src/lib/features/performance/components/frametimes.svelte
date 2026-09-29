<script lang="ts">
    import { onMount } from "svelte";
    import { Play, Square } from "@lucide/svelte";

    import Button from "$lib/components/ui/button.svelte";
    import Input from "$lib/components/ui/input.svelte";

    import { frameCapture } from "$lib/features/performance/frames.svelte";
    import { addonTitle, formatDuration, framePolyline } from "$lib/features/performance/performance";
    import { savedRuns } from "$lib/features/performance/runs.svelte";
    import { performanceScan } from "$lib/features/performance/scan.svelte";

    const GRAPH_W = 600;
    const GRAPH_H = 120;
    const MIN_CEILING_MS = 33.3;

    const status = $derived(frameCapture.status);
    const result = $derived(frameCapture.result);
    const recent = $derived(status?.recentFrametimesMs ?? []);
    const ceiling = $derived(Math.max(MIN_CEILING_MS, ...recent));
    const noFrames = $derived(status?.state === "capturing" && status.frames === 0 && status.elapsedMs > 5000);

    let label = $state("");
    let savedResult = $state<typeof result>(null);
    const enabledAddons = $derived(
        performanceScan.addons
            .filter((a) => a.enabled !== false)
            .map((a) => addonTitle(a, performanceScan.scans[a.fileName])),
    );
    const saved = $derived(result !== null && savedResult === result);

    async function saveRun() {
        if (!result) return;
        const snapshot = result;
        if (await savedRuns.save(label, enabledAddons, snapshot)) {
            savedResult = snapshot;
            label = "";
        }
    }

    onMount(() => void savedRuns.load());

    const ms = (v: number) => v.toFixed(2);
    const fps = (v: number) => v.toFixed(0);
</script>

{#snippet stat(label: string, value: string)}
    <div class="rounded-md border border-border bg-card px-4 py-3">
        <p class="font-heading text-2xl font-semibold">{value}</p>
        <p class="text-xs text-muted-foreground">{label}</p>
    </div>
{/snippet}

<div class="flex flex-col gap-4">
    <div class="flex items-start justify-between gap-4">
        <p class="text-sm text-muted-foreground">
            Records how evenly Deadlock presents frames. Start it, play, then stop to see the numbers. Run it with a mod
            on and again with it off to compare. Needs the app to run as administrator.
        </p>
        {#if frameCapture.active}
            <Button variant="outline" size="sm" onclick={() => frameCapture.stop()}><Square /> Stop</Button>
        {:else}
            <Button size="sm" onclick={() => frameCapture.start()}><Play /> Start</Button>
        {/if}
    </div>

    {#if frameCapture.error}
        <p class="text-sm text-destructive">{frameCapture.error}</p>
    {/if}
    {#if status?.state === "failed"}
        <p class="text-sm text-destructive">{status.error}</p>
    {/if}

    {#if frameCapture.active && status}
        <div class="flex flex-col gap-3 rounded-md border border-border bg-card px-4 py-3">
            <div class="flex items-center justify-between text-sm">
                <span>
                    {#if status.state === "waitingForGame"}
                        Waiting for Deadlock to start...
                    {:else}
                        Recording · {formatDuration(status.elapsedMs)} · {status.frames} frames
                    {/if}
                </span>
                {#if !status.gameFocused && status.state === "capturing"}
                    <span class="text-xs text-warning">Deadlock is in the background, not counting frames</span>
                {/if}
                {#if status.truncated}<span class="text-xs text-warning">Frame limit reached</span>{/if}
            </div>
            {#if recent.length > 1}
                <svg viewBox="0 0 {GRAPH_W} {GRAPH_H}" class="h-32 w-full" preserveAspectRatio="none">
                    <polyline
                        points={framePolyline(recent, GRAPH_W, GRAPH_H, ceiling)}
                        fill="none"
                        stroke="currentColor"
                        class="text-brass"
                        stroke-width="1.5"
                        vector-effect="non-scaling-stroke"
                    />
                </svg>
                <p class="text-xs text-muted-foreground">
                    Last {recent.length} frames · top of graph {ceiling.toFixed(0)} ms
                </p>
            {/if}
            {#if noFrames}
                <p class="text-sm text-warning">
                    Deadlock is running but no frames are arriving. Its renderer may not report frames this way.
                </p>
            {/if}
        </div>
    {/if}

    {#if result}
        {#if result.frameCount === 0}
            <p class="text-sm text-muted-foreground">No frames were recorded.</p>
        {:else}
            <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
                {@render stat("Average FPS", fps(1000 / result.avgMs))}
                {@render stat("1% low FPS", fps(result.low1pctFps))}
                {@render stat("0.1% low FPS", fps(result.low01pctFps))}
                {@render stat("Spikes", String(result.spikes.length))}
                {@render stat("Median frametime", `${ms(result.medianMs)} ms`)}
                {@render stat("95th percentile", `${ms(result.p95Ms)} ms`)}
                {@render stat("99th percentile", `${ms(result.p99Ms)} ms`)}
                {@render stat("Longest frame", `${ms(result.maxMs)} ms`)}
            </div>
            <p class="text-xs text-muted-foreground">
                {result.frameCount} frames over {formatDuration(result.durationMs)}. A spike is a frame at least 33 ms
                long and 2.5 times the median.
                {#if result.backgroundMs >= 1000}
                    Left out {formatDuration(result.backgroundMs)} spent tabbed out.
                {/if}
            </p>
            <div class="flex flex-col gap-2 rounded-md border border-border bg-card px-4 py-3">
                <p class="text-sm">
                    Save this run to compare later. It records the {enabledAddons.length} addon{enabledAddons.length ===
                    1
                        ? ""
                        : "s"} that are on now.
                </p>
                <div class="flex gap-2">
                    <Input bind:value={label} placeholder="Label, e.g. Mod off" maxlength={60} disabled={saved} />
                    <Button size="sm" onclick={saveRun} disabled={saved}>{saved ? "Saved" : "Save run"}</Button>
                </div>
                {#if savedRuns.error}<p class="text-sm text-destructive">{savedRuns.error}</p>{/if}
            </div>
            {#if result.spikes.length > 0}
                <ul class="flex flex-col divide-y divide-border rounded-md border border-border bg-card text-sm">
                    {#each result.spikes.slice(0, 50) as spike}
                        <li class="flex justify-between px-4 py-2">
                            <span class="font-mono">{ms(spike.frametimeMs)} ms</span>
                            <span class="text-muted-foreground">at {formatDuration(spike.atMs)}</span>
                        </li>
                    {/each}
                </ul>
            {/if}
        {/if}
    {/if}
</div>
