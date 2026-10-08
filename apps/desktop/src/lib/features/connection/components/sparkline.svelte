<script lang="ts">
    import { formatNumber, t } from "$lib/core/i18n.svelte";
    import { seriesBounds, sparkPath } from "../sparkline";

    type Series = { label: string; color: string; values: (number | null)[] };

    let { series, height = 120 }: { series: Series[]; height?: number } = $props();

    const width = 600;
    const pad = 4;

    const bounds = $derived(seriesBounds(series.map((s) => s.values)));
    const geometry = $derived({ width, height, pad, min: bounds.min, max: bounds.max });
</script>

<div class="relative">
    <svg
        viewBox="0 0 {width} {height}"
        class="h-auto w-full"
        preserveAspectRatio="none"
        role="img"
        aria-label={t("connection.history.chart_label")}
    >
        {#each series as s (s.label)}
            <path
                d={sparkPath(s.values, geometry)}
                fill="none"
                stroke={s.color}
                stroke-width="2"
                vector-effect="non-scaling-stroke"
            />
        {/each}
    </svg>
    <span class="absolute left-1 top-0 text-[10px] text-muted-foreground"
        >{t("connection.ms", { value: formatNumber(bounds.max) })}</span
    >
    <span class="absolute bottom-0 left-1 text-[10px] text-muted-foreground"
        >{t("connection.ms", { value: formatNumber(bounds.min) })}</span
    >
</div>
