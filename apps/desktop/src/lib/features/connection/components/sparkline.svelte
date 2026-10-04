<script lang="ts">
    import { formatNumber, t } from "$lib/core/i18n.svelte";

    type Series = { label: string; color: string; values: (number | null)[] };

    let { series, height = 120 }: { series: Series[]; height?: number } = $props();

    const width = 600;
    const pad = 4;

    const bounds = $derived.by(() => {
        const all = series.flatMap((s) => s.values.filter((v): v is number => v !== null));
        if (all.length === 0) return { min: 0, max: 1 };
        const min = Math.floor(Math.min(...all) - 5);
        const max = Math.ceil(Math.max(...all) + 5);
        return { min: Math.max(0, min), max: Math.max(max, min + 10) };
    });

    function path(values: (number | null)[]): string {
        const n = Math.max(values.length - 1, 1);
        let d = "";
        let pen = false;
        values.forEach((v, i) => {
            if (v === null) {
                pen = false;
                return;
            }
            const x = pad + (i / n) * (width - pad * 2);
            const y = pad + (1 - (v - bounds.min) / (bounds.max - bounds.min)) * (height - pad * 2);
            d += `${pen ? "L" : "M"}${x.toFixed(1)} ${y.toFixed(1)} `;
            pen = true;
        });
        return d;
    }
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
            <path d={path(s.values)} fill="none" stroke={s.color} stroke-width="2" vector-effect="non-scaling-stroke" />
        {/each}
    </svg>
    <span class="absolute left-1 top-0 text-[10px] text-muted-foreground"
        >{t("connection.ms", { value: formatNumber(bounds.max) })}</span
    >
    <span class="absolute bottom-0 left-1 text-[10px] text-muted-foreground"
        >{t("connection.ms", { value: formatNumber(bounds.min) })}</span
    >
</div>
