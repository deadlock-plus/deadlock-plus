<script lang="ts">
    import { ArrowDown, ArrowUp } from "@lucide/svelte";

    import Card from "$lib/ui/card.svelte";
    import type { RankChange, RankTier } from "../../rank";
    import { shortDay } from "../../format";
    import { badgeName } from "../../rank-view";

    let { changes, ranks }: { changes: RankChange[]; ranks: RankTier[] } = $props();
</script>

<section>
    <h2 class="mb-2 text-xl">Rank changes</h2>
    {#if changes.length === 0}
        <p class="text-sm text-muted-foreground">No badge changes in the history the API holds.</p>
    {:else}
        <ul class="flex flex-col gap-2">
            {#each changes as c (c.matchId)}
                <Card as="li" radius="md" padding="none" class="flex h-14 items-center gap-3 px-3 text-sm">
                    {#if c.promoted}
                        <ArrowUp class="size-4 text-primary" aria-label="Promoted" />
                    {:else}
                        <ArrowDown class="size-4 text-destructive" aria-label="Demoted" />
                    {/if}
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-medium">{badgeName(ranks, c.to)}</p>
                        <p class="text-xs text-muted-foreground">
                            {c.promoted ? "Promoted from" : "Demoted from"}
                            {badgeName(ranks, c.from)} · {shortDay(c.startTime)}
                        </p>
                    </div>
                </Card>
            {/each}
        </ul>
    {/if}
</section>
