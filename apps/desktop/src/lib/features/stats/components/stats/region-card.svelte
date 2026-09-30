<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import { LEADERBOARD_RULES, type LeaderboardProgress } from "../../leaderboard";
    import NeedBar from "./need-bar.svelte";

    let { board }: { board: LeaderboardProgress } = $props();

    const rule = LEADERBOARD_RULES.region;
</script>

<Card as="section">
    <div class="flex flex-wrap items-baseline justify-between gap-2">
        <h2 class="text-xl">Region leaderboard</h2>
        <span class="text-sm {board.regionReady ? 'text-primary' : 'text-muted-foreground'}">
            {board.regionReady ? "Eligible" : "Not yet eligible"}
        </span>
    </div>
    <p class="mt-1 text-sm text-muted-foreground">
        Approximate. Counts your ranked and unranked games known to the API. Valve does not publish which queues count.
    </p>
    <div class="mt-3 grid gap-4 sm:grid-cols-2">
        <NeedBar label="Games in the last {rule.windowDays} days" value={board.recentGames} need={rule.gamesInWindow} />
        <NeedBar label="Total games on your account" value={board.totalGames} need={rule.totalGames} />
    </div>
</Card>
