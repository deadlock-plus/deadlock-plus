<script lang="ts">
    import type { Match, Streaks, WinLoss } from "../../stats";
    import { formatPlaytime, totalPlaytime } from "../../stats";
    import { pct } from "../../format";
    import StatTile from "../shared/stat-tile.svelte";
    import SyncingBadge from "../shared/syncing-badge.svelte";

    let { windowed, summary, run, form }: { windowed: Match[]; summary: WinLoss; run: Streaks; form: Match[] } =
        $props();
    const syncing = $derived(windowed.filter((m) => m.provisional).length);
</script>

<div class="grid gap-3 sm:grid-cols-3">
    <StatTile label="Winrate" value={pct(summary.winrate)}>
        <p class="text-sm text-muted-foreground">
            {summary.wins}W {summary.losses}L{summary.unscored > 0 ? `, ${summary.unscored} not scored` : ""}
        </p>
        <div class="mt-3 flex gap-1" role="img" aria-label="Last {form.length} games, oldest first">
            {#each form as m (m.matchId)}
                <span
                    class="h-2 flex-1 rounded-full {m.outcome === 'win' ? 'bg-primary' : 'bg-destructive'}"
                    title={m.outcome === "win" ? "Win" : "Loss"}
                ></span>
            {/each}
        </div>
    </StatTile>
    <StatTile
        label="Streak"
        value={run.current ? `${run.current.length} ${run.current.kind === "win" ? "wins" : "losses"}` : "-"}
    >
        <p class="text-sm text-muted-foreground">Best {run.longestWin} wins, worst {run.longestLoss} losses</p>
    </StatTile>
    <StatTile label="Playtime" value={formatPlaytime(totalPlaytime(windowed))}>
        <p class="flex items-center gap-2 text-sm text-muted-foreground">
            {windowed.length} matches
            {#if syncing > 0}<SyncingBadge count={syncing} />{/if}
        </p>
    </StatTile>
</div>
