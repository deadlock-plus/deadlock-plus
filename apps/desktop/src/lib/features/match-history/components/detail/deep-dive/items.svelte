<script lang="ts">
    import { i18n, t } from "$lib/core/i18n.svelte";
    import { formatClock } from "$lib/features/live/live";
    import { resolveItemListVisuals, type IdVisual } from "../../../deep-dive/catalog";
    import { itemBars } from "../../../deep-dive/items";
    import type { MatchDetail, MatchPlayer } from "../../../detail";
    import IdTile from "./id-tile.svelte";
    import SectionCard from "./section-card.svelte";

    let { detail, selected }: { detail: MatchDetail; selected: MatchPlayer } = $props();

    const allBars = $derived(itemBars(selected.items, detail.durationS));
    const clock = (s: number) => formatClock(Math.round(s)) ?? "";

    let visuals = $state.raw(new Map<number, IdVisual>());
    let loaded = $state(false);
    let token = 0;

    // Ability rows (levelled ability upgrades repeat as item rows) are not items; hold the card back
    // until the catalog has said which rows those are.
    const bars = $derived(loaded ? allBars.filter((b) => visuals.get(b.itemId)?.kind !== "ability") : []);

    $effect(() => {
        const ids = [...new Set(allBars.map((b) => b.itemId))];
        const locale = i18n.locale;
        const mine = ++token;
        void resolveItemListVisuals(ids, locale).then((found) => {
            if (mine !== token) return;
            visuals = found;
            loaded = true;
        });
    });

    const nameOf = (id: number) => visuals.get(id)?.name ?? t("match_history.deep_dive.names.item");
</script>

<SectionCard title={t("match_history.deep_dive.items.heading")} available={(loaded ? bars : allBars).length > 0}>
    <ul class="flex flex-col gap-1.5">
        {#each bars as b, i (i)}
            <li class="grid grid-cols-[minmax(0,12rem)_1fr] items-center gap-3 text-sm">
                <span class="flex min-w-0 items-center gap-2">
                    <IdTile src={visuals.get(b.itemId)?.src} label={nameOf(b.itemId)} size={24} />
                    <span class="truncate">{nameOf(b.itemId)}</span>
                </span>
                <span class="flex flex-col gap-0.5">
                    <span class="relative h-2 rounded-full bg-muted">
                        <span
                            class="absolute inset-y-0 rounded-full {b.sold ? 'bg-muted-foreground' : 'bg-primary'}"
                            style:left="{b.left}%"
                            style:width="{b.width}%"
                        ></span>
                    </span>
                    <span class="text-[11px] text-muted-foreground tabular-nums">
                        {b.sold
                            ? t("match_history.deep_dive.items.bought_sold", {
                                  bought: clock(b.startS),
                                  sold: clock(b.endS),
                              })
                            : t("match_history.deep_dive.items.bought_held", { bought: clock(b.startS) })}
                    </span>
                </span>
            </li>
        {/each}
    </ul>
</SectionCard>
