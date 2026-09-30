<script lang="ts">
    import type { LossOutcome, WindowStats } from "../../rank";
    import { gainForWin } from "../../rank";
    import { pct, signed } from "../../format";
    import StatTile from "../shared/stat-tile.svelte";

    let {
        form,
        formWindow,
        sampled,
        streak,
        nextLoss,
    }: {
        form: WindowStats;
        formWindow: number;
        sampled: number;
        streak: number;
        nextLoss: LossOutcome | null;
    } = $props();

    const unscored = $derived(sampled - form.games);
</script>

<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
    <StatTile label="Last {formWindow} ranked" value={pct(form.winrate)}>
        <p class="text-sm text-muted-foreground">
            {form.wins}W {form.losses}L{unscored > 0 ? `, ${unscored} not scored` : ""}
        </p>
    </StatTile>
    <StatTile label="Progress, same games" value={signed(form.net)}>
        <p class="text-sm text-muted-foreground">1000 points is a subrank</p>
    </StatTile>
    <StatTile label="Next win" value="+{gainForWin(streak + 1)}">
        <p class="text-sm text-muted-foreground">
            {streak === 0 ? "No win streak" : `${streak} ${streak === 1 ? "win" : "wins"} in a row`}
        </p>
    </StatTile>
    <StatTile label="Next loss" value={nextLoss ? `-${nextLoss.lost}` : "-"}>
        {#if nextLoss}
            <p class="text-sm text-muted-foreground">
                {#if nextLoss.usesShield}
                    {nextLoss.lost === 0 ? "Only a shield" : "Plus a shield"}
                {:else if nextLoss.demotes}
                    No shield: drops a subrank
                {:else}
                    No shield used
                {/if}
            </p>
        {/if}
    </StatTile>
</div>
