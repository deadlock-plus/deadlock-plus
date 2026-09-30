<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { HeroStat } from "../../stats";
    import { pct } from "../../format";
    import HeroIcon from "./hero-icon.svelte";

    let { mostPlayed, best, heroes }: { mostPlayed: HeroStat; best: HeroStat | null; heroes: Record<number, Hero> } =
        $props();
</script>

<div class="grid gap-3 sm:grid-cols-2">
    <Card class="flex items-center gap-4">
        <HeroIcon hero={heroes[mostPlayed.heroId]} size="size-14" />
        <div class="min-w-0">
            <p class="text-sm text-muted-foreground">Most played</p>
            <p class="truncate font-heading text-xl">
                {heroes[mostPlayed.heroId]?.name ?? `Hero ${mostPlayed.heroId}`}
            </p>
            <p class="text-sm text-muted-foreground">
                {mostPlayed.games} games, {pct(mostPlayed.winrate)} winrate
            </p>
        </div>
    </Card>
    <Card class="flex items-center gap-4">
        {#if best}
            <HeroIcon hero={heroes[best.heroId]} size="size-14" />
            <div class="min-w-0">
                <p class="text-sm text-muted-foreground">Best winrate (5+ games)</p>
                <p class="truncate font-heading text-xl">{heroes[best.heroId]?.name ?? `Hero ${best.heroId}`}</p>
                <p class="text-sm text-muted-foreground">{pct(best.winrate)} over {best.games} games</p>
            </div>
        {:else}
            <p class="text-sm text-muted-foreground">Play 5 games on a hero to see your best winrate.</p>
        {/if}
    </Card>
</div>
