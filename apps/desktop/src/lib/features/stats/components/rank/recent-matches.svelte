<script lang="ts">
    import { CircleHelp, Shield } from "@lucide/svelte";

    import Card from "$lib/ui/card.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { RankPoint } from "../../rank";
    import { shortDay, signed } from "../../format";

    let { recent, heroes }: { recent: RankPoint[]; heroes: Record<number, Hero> } = $props();
</script>

<section>
    <h2 class="mb-2 text-xl">Recent ranked matches</h2>
    <ul class="flex flex-col gap-2">
        {#each recent as p (p.matchId)}
            {@const hero = heroes[p.heroId]}
            <Card as="li" radius="md" padding="none" class="flex h-14 items-center gap-3 px-3">
                <div class="flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
                    {#if hero?.icon}
                        <img src={hero.icon} alt="" class="size-full object-cover" />
                    {:else}
                        <CircleHelp class="size-4 text-muted-foreground" aria-label="Unknown hero" />
                    {/if}
                </div>
                <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium">{hero?.name ?? `Hero ${p.heroId}`}</p>
                    <p class="text-xs text-muted-foreground">{shortDay(p.startTime)}</p>
                </div>
                {#if p.demotionProtected}
                    <Shield class="size-4 fill-primary/25 text-primary" aria-label="A shield absorbed this loss" />
                {/if}
                <span
                    class="w-12 text-right text-sm {p.outcome === 'win'
                        ? 'text-primary'
                        : p.outcome === 'loss'
                          ? 'text-destructive'
                          : 'text-muted-foreground'}"
                >
                    {p.outcome === "win" ? "Win" : p.outcome === "loss" ? "Loss" : "-"}
                </span>
                <span class="w-14 text-right text-sm tabular-nums">{p.delta === null ? "-" : signed(p.delta)}</span>
            </Card>
        {/each}
    </ul>
</section>
