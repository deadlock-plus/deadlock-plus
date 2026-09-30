<script lang="ts">
    import Badge from "$lib/ui/badge.svelte";
    import Section from "$lib/ui/section.svelte";
    import Flag from "$lib/components/flag.svelte";
    import { gapVariant } from "../connection";
    import type { RelayInfo } from "../types";

    type Props = {
        gameRunning: boolean;
        relay: RelayInfo | null;
    };

    let { gameRunning, relay }: Props = $props();
</script>

<Section title="Current server">
    {#if !gameRunning}
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
                        <Badge variant={gapVariant(relay.maxGapMs)}>{Math.round(relay.maxGapMs)} ms</Badge>
                    </dd>
                </div>
            </dl>
        </div>
    {/if}
</Section>
