<script lang="ts">
    import { t } from "$lib/core/i18n.svelte";
    import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
    import Select from "$lib/ui/select.svelte";
    import { allPlayers, findPlayer, type MatchDetail } from "../../detail";
    import Abilities from "./deep-dive/abilities.svelte";
    import Accolades from "./deep-dive/accolades.svelte";
    import Damage from "./deep-dive/damage.svelte";
    import Deaths from "./deep-dive/deaths.svelte";
    import HeroChip from "./deep-dive/hero-chip.svelte";
    import Items from "./deep-dive/items.svelte";
    import Objectives from "./deep-dive/objectives.svelte";
    import ProgressCharts from "./deep-dive/progress-charts.svelte";

    let { detail, ownAccountIds = [] }: { detail: MatchDetail; ownAccountIds?: number[] } = $props();

    const players = $derived(allPlayers(detail));
    let pickedSlot = $state<number | null>(null);
    const selected = $derived(
        players.find((p) => p.slot === pickedSlot) ?? findPlayer(detail, ownAccountIds) ?? players[0],
    );

    let heroes = $state.raw<Record<number, Hero>>({});
    $effect(() => {
        const ids = players.map((p) => p.heroId);
        let live = true;
        void loadHeroes(ids).then((h) => {
            if (live) heroes = h;
        });
        return () => (live = false);
    });

    const label = (p: (typeof players)[number]) =>
        `${p.name ?? t("match_history.deep_dive.player_fallback", { slot: p.slot })} · ${heroes[p.heroId]?.name ?? p.heroId}`;
</script>

<div class="dd flex flex-col gap-4" data-match-id={detail.matchId}>
    {#if selected}
        <label class="flex flex-wrap items-center gap-2 text-sm">
            <span class="text-muted-foreground">{t("match_history.deep_dive.player")}</span>
            <HeroChip heroId={selected.heroId} {heroes} team={selected.team} size={26} emphasised />
            <Select value={selected.slot} onchange={(e) => (pickedSlot = Number(e.currentTarget.value))}>
                {#each players as p (p.slot)}
                    <option value={p.slot}>{label(p)}</option>
                {/each}
            </Select>
        </label>

        <ProgressCharts {detail} {selected} {heroes} />
        <Deaths {detail} {selected} {heroes} />
        <Items {detail} {selected} />
        <Damage {detail} selectedSlot={selected.slot} {heroes} />
        <Objectives {detail} />
        <Abilities {selected} />
        <Accolades {selected} />
    {/if}
</div>

<style>
    .dd {
        --dd-hk: oklch(0.8 0.125 82);
        --dd-am: oklch(0.72 0.11 245);
    }
    :global(:root[data-theme="daylight"]) .dd {
        --dd-hk: oklch(0.6 0.12 70);
        --dd-am: oklch(0.5 0.12 245);
    }
</style>
