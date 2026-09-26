<script lang="ts">
    import Badge from "$lib/components/ui/badge.svelte";
    import type { PingStats } from "../types";

    type Props = {
        title: string;
        note: string;
        stats: PingStats | null;
        offset?: number;
        unavailable?: string;
        estimate?: boolean;
    };

    let { title, note, stats, offset = 0, unavailable, estimate = false }: Props = $props();

    const fmt = (v: number | null | undefined) => (v == null ? "—" : Math.round(v + offset).toString());
    const headline = $derived(stats ? (stats.current ?? stats.avg) : null);
</script>

<div class="flex flex-col gap-3 rounded-lg border border-border bg-card p-4">
    <div class="flex items-center justify-between">
        <span class="text-sm font-medium">{title}</span>
        {#if estimate}<Badge variant="warning">estimate</Badge>{/if}
    </div>

    {#if unavailable || !stats}
        <p class="py-6 text-sm text-muted-foreground">{unavailable ?? "No data yet."}</p>
    {:else}
        <div class="flex items-baseline gap-1">
            <span class="text-4xl font-semibold tabular-nums">{fmt(headline)}</span>
            <span class="text-sm text-muted-foreground">ms</span>
        </div>
        <dl class="grid grid-cols-4 gap-2 text-xs">
            <div>
                <dt class="text-muted-foreground">avg</dt>
                <dd class="tabular-nums">{fmt(stats.avg)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">min</dt>
                <dd class="tabular-nums">{fmt(stats.min)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">max</dt>
                <dd class="tabular-nums">{fmt(stats.max)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">jitter</dt>
                <dd class="tabular-nums">{stats.jitter == null ? "—" : stats.jitter.toFixed(1)}</dd>
            </div>
        </dl>
        <div class="flex items-center gap-2 text-xs">
            <span class="text-muted-foreground">loss</span>
            <Badge variant={stats.lossPct === 0 ? "success" : stats.lossPct < 3 ? "warning" : "destructive"}>
                {stats.lossPct.toFixed(stats.lossPct === 0 ? 0 : 1)}%
            </Badge>
            <span class="text-muted-foreground">over {stats.samples} pings</span>
        </div>
    {/if}

    <p class="text-xs text-muted-foreground">{note}</p>
</div>
