<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { HeroRow } from "../../hero-rows";
    import { LEADERBOARD_RULES } from "../../leaderboard";
    import { formatPlaytime } from "../../stats";
    import { pct } from "../../format";
    import HeroIcon from "./hero-icon.svelte";
    import NeedBar from "./need-bar.svelte";

    let { rows, heroes }: { rows: HeroRow[]; heroes: Record<number, Hero> } = $props();

    const rule = LEADERBOARD_RULES.hero;
</script>

<section>
    <h2 class="text-xl">Heroes</h2>
    <p class="mb-3 mt-1 text-sm text-muted-foreground">
        Stats follow the filters above. Hero leaderboard progress is always the last {rule.windowDays} days plus lifetime
        wins, and also needs {rule.totalGames} total games on your account.
    </p>
    <ul class="flex flex-col gap-3">
        {#each rows as row (row.heroId)}
            {@const hero = heroes[row.heroId]}
            {@const st = row.stat}
            <Card as="li">
                <div class="flex items-center gap-4">
                    <HeroIcon {hero} size="size-16" />
                    <div class="min-w-0 flex-1">
                        <div class="flex items-baseline justify-between gap-3">
                            <p class="truncate font-heading text-xl">{hero?.name ?? `Hero ${row.heroId}`}</p>
                            <p class="shrink-0 text-sm {row.board.ready ? 'text-primary' : 'text-muted-foreground'}">
                                {row.board.ready ? "Hero board eligible" : "Hero board not yet"}
                            </p>
                        </div>
                        {#if st}
                            <div class="mt-2 flex items-center gap-3">
                                <div class="h-2 flex-1 overflow-hidden rounded-full bg-destructive/60">
                                    <div class="h-full bg-primary" style:width={pct(st.winrate)}></div>
                                </div>
                                <span class="w-10 text-right text-sm font-medium">{pct(st.winrate)}</span>
                            </div>
                            <dl class="mt-2 flex flex-wrap gap-x-8 gap-y-1 text-sm">
                                <div>
                                    <dt class="text-xs text-muted-foreground">Games</dt>
                                    <dd>{st.games}</dd>
                                </div>
                                <div>
                                    <dt class="text-xs text-muted-foreground">KDA</dt>
                                    <dd>{st.kda.toFixed(2)}</dd>
                                </div>
                                <div>
                                    <dt class="text-xs text-muted-foreground">Avg souls</dt>
                                    <dd>{Math.round(st.avgNetWorth).toLocaleString()}</dd>
                                </div>
                                <div>
                                    <dt class="text-xs text-muted-foreground">Time</dt>
                                    <dd>{formatPlaytime(st.playtimeS)}</dd>
                                </div>
                            </dl>
                        {:else if row.lifetimeGames > 0}
                            <p class="mt-2 text-sm text-muted-foreground">
                                No games in this selection. {row.lifetimeGames} lifetime games.
                            </p>
                        {:else}
                            <p class="mt-2 text-sm text-muted-foreground">No games in this selection.</p>
                        {/if}
                    </div>
                </div>
                <div class="mt-4 grid gap-4 border-t border-border pt-3 sm:grid-cols-2">
                    <NeedBar
                        label="Games, last {rule.windowDays} days"
                        value={row.board.recentGames}
                        need={rule.gamesInWindow}
                    />
                    <NeedBar label="Lifetime wins" value={row.board.wins} need={rule.lifetimeWins} />
                </div>
            </Card>
        {/each}
    </ul>
</section>
