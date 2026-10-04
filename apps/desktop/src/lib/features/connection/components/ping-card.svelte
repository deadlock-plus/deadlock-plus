<script lang="ts">
    import Badge from "$lib/ui/badge.svelte";
    import Card from "$lib/ui/card.svelte";
    import { formatNumber, t, tn } from "$lib/core/i18n.svelte";
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

    const fmt = (v: number | null | undefined) => (v == null ? "—" : formatNumber(Math.round(v + offset)));
    const decimal = (v: number) => formatNumber(v, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
    const headline = $derived(stats ? (stats.current ?? stats.avg) : null);
</script>

<Card class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
        <span class="text-sm font-medium">{title}</span>
        {#if estimate}<Badge variant="warning">{t("connection.ping.estimate")}</Badge>{/if}
    </div>

    {#if unavailable || !stats}
        <p class="py-6 text-sm text-muted-foreground">{unavailable ?? t("connection.ping.no_data")}</p>
    {:else}
        <div class="flex items-baseline gap-1">
            <span class="text-4xl font-semibold tabular-nums">{fmt(headline)}</span>
            <span class="text-sm text-muted-foreground">{t("connection.unit_ms")}</span>
        </div>
        <dl class="grid grid-cols-4 gap-2 text-xs">
            <div>
                <dt class="text-muted-foreground">{t("connection.ping.avg")}</dt>
                <dd class="tabular-nums">{fmt(stats.avg)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">{t("connection.ping.min")}</dt>
                <dd class="tabular-nums">{fmt(stats.min)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">{t("connection.ping.max")}</dt>
                <dd class="tabular-nums">{fmt(stats.max)}</dd>
            </div>
            <div>
                <dt class="text-muted-foreground">{t("connection.ping.jitter")}</dt>
                <dd class="tabular-nums">{stats.jitter == null ? "—" : decimal(stats.jitter)}</dd>
            </div>
        </dl>
        <div class="flex items-center gap-2 text-xs">
            <span class="text-muted-foreground">{t("connection.ping.loss")}</span>
            <Badge variant={stats.lossPct === 0 ? "success" : stats.lossPct < 3 ? "warning" : "destructive"}>
                {formatNumber(stats.lossPct, {
                    minimumFractionDigits: 0,
                    maximumFractionDigits: stats.lossPct === 0 ? 0 : 1,
                })}%
            </Badge>
            <span class="text-muted-foreground">{tn("connection.ping.samples", stats.samples)}</span>
        </div>
    {/if}

    <p class="text-xs text-muted-foreground">{note}</p>
</Card>
