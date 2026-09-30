<script lang="ts">
    import Badge from "$lib/ui/badge.svelte";
    import Section from "$lib/ui/section.svelte";
    import type { EndpointInfo } from "../types";

    type Props = {
        endpoints: EndpointInfo[];
    };

    let { endpoints }: Props = $props();
</script>

<Section title="ExitLag path" titleClass="mb-2">
    <div class="flex flex-col gap-1.5">
        {#each endpoints as e (e.ip + e.port)}
            <div class="flex items-center gap-3 text-xs">
                <Badge variant={e.isExit ? "success" : "secondary"}>{e.isExit ? "exit server" : "entry point"}</Badge>
                <span class="font-mono">{e.ip}:{e.port}</span>
                <span class="text-muted-foreground tabular-nums">{Math.round(e.pps)} pkt/s</span>
                <span class="ml-auto tabular-nums">{e.ping.avg == null ? "—" : `${Math.round(e.ping.avg)} ms`}</span>
            </div>
        {/each}
    </div>
</Section>
