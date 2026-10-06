<script lang="ts">
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { RankTier } from "$lib/features/stats/rank";
    import type { LiveMatch } from "$lib/generated/types/LiveMatch";
    import type { LivePhase } from "$lib/generated/types/LivePhase";
    import { orderTeams } from "../live";
    import MatchHeader from "./match-header.svelte";
    import TeamTable from "./team-table.svelte";

    let {
        match,
        phase,
        heroes,
        tiers,
    }: { match: LiveMatch; phase: LivePhase; heroes: Record<number, Hero>; tiers: RankTier[] } = $props();

    const teams = $derived(orderTeams(match.teams, match.yourSide));
    const totalSouls = $derived(match.teams.reduce((sum, t) => sum + t.souls, 0));
</script>

<div class="flex flex-col gap-3">
    <MatchHeader {match} {phase} />
    {#each teams as team (team.side)}
        <TeamTable {team} {totalSouls} clockSecs={match.clockSecs} pregame={phase === "pregame"} {heroes} {tiers} />
    {/each}
</div>
