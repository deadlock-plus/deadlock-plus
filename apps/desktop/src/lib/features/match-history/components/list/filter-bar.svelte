<script lang="ts">
    import { Users, X } from "@lucide/svelte";

    import { t, tn } from "$lib/core/i18n.svelte";
    import Select from "$lib/ui/select.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { RowFilters } from "../../filters";
    import Segmented from "./segmented.svelte";
    import { DAY_OPTIONS, MODE_OPTIONS, OUTCOME_OPTIONS, filtersActive, type HeroOption } from "./view-model";

    let {
        filters,
        heroes,
        heroArt,
        onchange,
        onreset,
    }: {
        filters: RowFilters;
        heroes: HeroOption[];
        heroArt: Record<number, Hero>;
        onchange: (patch: Partial<RowFilters>) => void;
        onreset: () => void;
    } = $props();

    const modes = $derived(MODE_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) })));
    const outcomes = $derived(OUTCOME_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) })));
    const windows = $derived(
        DAY_OPTIONS.map((d) => ({
            value: d === null ? "" : String(d),
            label: d === null ? t("stats.filter.all_time") : tn("stats.filter.days", d),
        })),
    );
    const pickedIcon = $derived(filters.heroId === null ? undefined : heroArt[filters.heroId]?.icon);
</script>

<div
    class="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-lg border border-border bg-card px-3 py-2.5"
    role="group"
    aria-label={t("match_history.list.filter.label")}
>
    <Segmented
        label={t("match_history.list.filter.mode")}
        options={modes}
        value={filters.mode}
        disabled={filters.custom}
        onselect={(v) => onchange({ mode: v })}
    />
    <Segmented
        label={t("match_history.list.filter.outcome")}
        options={outcomes}
        value={filters.outcome}
        onselect={(v) => onchange({ outcome: v })}
    />
    <Segmented
        label={t("match_history.list.filter.window")}
        options={windows}
        value={filters.days === null ? "" : String(filters.days)}
        onselect={(v) => onchange({ days: v === "" ? null : Number(v) })}
    />
    <div class="relative">
        <span
            class="pointer-events-none absolute top-1/2 left-1.5 flex size-6 -translate-y-1/2 items-center justify-center overflow-hidden rounded bg-muted"
        >
            {#if pickedIcon}
                <img src={pickedIcon} alt="" class="size-full object-cover" />
            {:else}
                <Users class="size-3.5 text-muted-foreground" aria-hidden="true" />
            {/if}
        </span>
        <Select
            class="min-w-44 pl-9"
            aria-label={t("match_history.list.filter.hero")}
            value={filters.heroId === null ? "" : String(filters.heroId)}
            onchange={(e) => {
                const v = e.currentTarget.value;
                onchange({ heroId: v === "" ? null : Number(v) });
            }}
        >
            <option value="">{t("match_history.list.filter.any_hero")}</option>
            {#each heroes as h (h.id)}
                <option value={String(h.id)}>{h.name}</option>
            {/each}
        </Select>
    </div>
    <button
        type="button"
        aria-pressed={filters.custom}
        class="inline-flex h-9 items-center gap-1.5 rounded-md border px-3 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 {filters.custom
            ? 'border-transparent bg-primary text-primary-foreground'
            : 'border-input bg-card text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
        onclick={() => onchange({ custom: !filters.custom })}
    >
        {t("match_history.list.filter.custom")}
    </button>
    {#if filtersActive(filters)}
        <button
            type="button"
            class="ml-auto inline-flex h-9 items-center gap-1.5 rounded-md px-2 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
            onclick={onreset}
        >
            <X class="size-3.5" aria-hidden="true" />
            {t("match_history.list.filter.reset")}
        </button>
    {/if}
</div>
