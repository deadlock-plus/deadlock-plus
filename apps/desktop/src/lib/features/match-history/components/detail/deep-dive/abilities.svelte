<script lang="ts">
    import { formatNumber, i18n, t } from "$lib/core/i18n.svelte";
    import { resolveVisuals, type IdVisual } from "../../../deep-dive/catalog";
    import type { MatchPlayer } from "../../../detail";
    import IdTile from "./id-tile.svelte";
    import SectionCard from "./section-card.svelte";

    let { selected }: { selected: MatchPlayer } = $props();

    let visuals = $state.raw(new Map<number, IdVisual>());
    let token = 0;

    $effect(() => {
        const ids = selected.abilities.map((a) => a.abilityId);
        const locale = i18n.locale;
        const mine = ++token;
        void resolveVisuals(ids, locale, "ability").then((found) => {
            if (mine === token) visuals = found;
        });
    });

    const nameOf = (id: number) => visuals.get(id)?.name ?? t("match_history.deep_dive.names.ability");
</script>

<SectionCard title={t("match_history.deep_dive.abilities.heading")} available={selected.abilities.length > 0}>
    <ul class="grid gap-2 sm:grid-cols-2">
        {#each selected.abilities as a, i (i)}
            <li class="flex items-center gap-2 text-sm">
                <IdTile src={visuals.get(a.abilityId)?.src} label={nameOf(a.abilityId)} />
                <span class="min-w-0 flex-1 truncate">{nameOf(a.abilityId)}</span>
                <span class="tabular-nums text-muted-foreground">{formatNumber(Math.round(a.value))}</span>
            </li>
        {/each}
    </ul>
</SectionCard>
