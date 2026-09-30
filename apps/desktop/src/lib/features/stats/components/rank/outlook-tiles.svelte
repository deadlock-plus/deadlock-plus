<script lang="ts">
    import type { Forecast } from "../../climb";
    import type { WindowStats } from "../../rank";
    import { etaText, pct, signed } from "../../format";
    import StatTile from "../shared/stat-tile.svelte";

    let {
        form,
        formWindow,
        breakEven,
        forecast,
        nextName,
    }: {
        form: WindowStats;
        formWindow: number;
        breakEven: number;
        forecast: Forecast | null;
        nextName: string;
    } = $props();
</script>

<div class="grid gap-3 md:grid-cols-2">
    <StatTile label="Winrate to hold your rank" value="{Math.round(breakEven * 100)}%">
        <p class="text-sm text-muted-foreground">
            {#if form.winrate === null}
                Win streaks pay more than losses cost, so you don't need 50%.
            {:else}
                You are at {pct(form.winrate)} over your last {formWindow}: {form.winrate >= breakEven
                    ? "climbing"
                    : "sliding"}.
            {/if}
        </p>
    </StatTile>
    <StatTile label="Climb forecast" value={forecast ? `${signed(Math.round(forecast.perDay))} a day` : "-"}>
        <p class="text-sm text-muted-foreground">
            {#if forecast}
                {nextName}: {etaText(forecast.subrank)}{forecast.tier ? `. Next tier: ${etaText(forecast.tier)}` : ""}.
            {:else}
                Needs about 8 ranked games in the last two weeks.
            {/if}
        </p>
    </StatTile>
</div>
