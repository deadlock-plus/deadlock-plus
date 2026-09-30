<script lang="ts">
    import { onMount } from "svelte";
    import { CircleHelp, LoaderCircle, RefreshCw } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { stats } from "$lib/features/stats/stats.svelte";
    import CachedNote from "$lib/features/stats/components/shared/cached-note.svelte";
    import { LEADERBOARD_RULES, leaderboardProgress } from "$lib/features/stats/leaderboard";
    import {
        filterScope,
        formatPlaytime,
        heroBreakdown,
        inWindow,
        record,
        streaks,
        totalPlaytime,
        type Scope,
    } from "$lib/features/stats/stats";

    const SCOPES: { id: Scope; label: string }[] = [
        { id: "ranked", label: "Ranked" },
        { id: "unranked", label: "Unranked" },
        { id: "all", label: "All modes" },
    ];
    const WINDOWS: { days: number | null; label: string }[] = [
        { days: 7, label: "7 days" },
        { days: 30, label: "30 days" },
        { days: 90, label: "90 days" },
        { days: null, label: "All time" },
    ];

    let scope = $state<Scope>("ranked");
    let days = $state<number | null>(30);
    let heroes = $state<Record<number, Hero>>({});

    const accountId = $derived(steamAccount.account?.steamId32 ?? null);
    const scoped = $derived(filterScope(stats.matches, scope));
    const now = $derived(Date.now() / 1000);
    const windowed = $derived(inWindow(scoped, days, now));
    const summary = $derived(record(windowed));
    const run = $derived(streaks(scoped));
    const perHero = $derived(heroBreakdown(windowed));
    const perHeroLifetime = $derived(heroBreakdown(scoped));
    const board = $derived(leaderboardProgress(stats.matches, now));
    const heroRows = $derived(
        board.heroes
            .map((b) => ({
                heroId: b.heroId,
                stat: perHero.find((r) => r.heroId === b.heroId) ?? null,
                lifetimeGames: perHeroLifetime.find((r) => r.heroId === b.heroId)?.games ?? 0,
                board: b,
            }))
            .sort(
                (a, b) =>
                    (b.stat?.games ?? 0) - (a.stat?.games ?? 0) ||
                    Number(b.board.ready) - Number(a.board.ready) ||
                    a.heroId - b.heroId,
            ),
    );
    const form = $derived(windowed.filter((m) => m.outcome !== "unscored").slice(-12));
    const mostPlayed = $derived(perHero[0] ?? null);
    const bestHero = $derived(
        perHero
            .filter((r) => r.games >= 5 && r.winrate !== null)
            .sort((a, b) => (b.winrate ?? 0) - (a.winrate ?? 0))[0] ?? null,
    );
    const { region: regionRule, hero: heroRule } = LEADERBOARD_RULES;
    const barPct = (value: number, need: number) => `${Math.min(100, (value / need) * 100)}%`;

    const pct = (v: number | null) => (v === null ? "-" : `${Math.round(v * 100)}%`);

    $effect(() => {
        if (accountId !== null) void stats.load(accountId);
    });

    onMount(() => {
        void loadHeroes().then((h) => (heroes = h));
    });
</script>

<Page size="lg">
    <PageHeader size="lg" title="Stats" subtitle="Worked out on this PC from your match history in the Deadlock API.">
        {#snippet actions()}
            <Button
                variant="outline"
                size="sm"
                disabled={accountId === null || stats.status === "loading"}
                onclick={() => accountId !== null && stats.load(accountId, true)}
            >
                <RefreshCw class={stats.status === "loading" ? "animate-spin" : ""} /> Refresh
            </Button>
        {/snippet}
    </PageHeader>

    {#if steamAccount.loaded && accountId === null}
        <EmptyState size="base" spacing="xl">No Steam account found, so there is no history to load.</EmptyState>
    {:else if stats.status === "error"}
        <EmptyState size="base" spacing="xl" tone="destructive"
            >Could not load your match history. {stats.error}</EmptyState
        >
    {:else if stats.status !== "ready"}
        <EmptyState size="base" spacing="xl" class="flex items-center justify-center gap-2">
            <LoaderCircle class="size-4 animate-spin" /> Loading your match history...
        </EmptyState>
    {:else}
        <CachedNote />
        <div class="flex flex-wrap items-center gap-2">
            {#each SCOPES as s (s.id)}
                <Button size="sm" variant={scope === s.id ? "default" : "outline"} onclick={() => (scope = s.id)}
                    >{s.label}</Button
                >
            {/each}
            <span class="mx-2 h-5 w-px bg-border" aria-hidden="true"></span>
            {#each WINDOWS as w (w.label)}
                <Button size="sm" variant={days === w.days ? "default" : "outline"} onclick={() => (days = w.days)}
                    >{w.label}</Button
                >
            {/each}
        </div>

        <p class="text-sm text-muted-foreground">
            {stats.matches.length} matches known to the API for this account. Anything it has not seen is not counted.
        </p>

        {#snippet bar(label: string, value: number, need: number)}
            <div>
                <div class="flex justify-between text-sm">
                    <span class="text-muted-foreground">{label}</span>
                    <span class={value >= need ? "text-primary" : ""}>{value} / {need}</span>
                </div>
                <div class="mt-1 h-2 overflow-hidden rounded-full bg-muted">
                    <div class="h-full rounded-full bg-primary" style:width={barPct(value, need)}></div>
                </div>
            </div>
        {/snippet}

        {#snippet heroIcon(heroId: number, size: string)}
            {@const hero = heroes[heroId]}
            <div class="flex shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted {size}">
                {#if hero?.icon}
                    <img src={hero.icon} alt="" class="size-full object-cover" />
                {:else}
                    <CircleHelp class="size-6 text-muted-foreground" aria-label="Unknown hero" />
                {/if}
            </div>
        {/snippet}

        {#if windowed.length === 0}
            <EmptyState size="base" spacing="md">No matches in this selection.</EmptyState>
        {:else}
            <div class="grid gap-3 sm:grid-cols-3">
                <Card>
                    <p class="text-sm text-muted-foreground">Winrate</p>
                    <p class="mt-1 font-heading text-3xl font-semibold">{pct(summary.winrate)}</p>
                    <p class="text-sm text-muted-foreground">
                        {summary.wins}W {summary.losses}L{summary.unscored > 0
                            ? `, ${summary.unscored} not scored`
                            : ""}
                    </p>
                    <div class="mt-3 flex gap-1" role="img" aria-label="Last {form.length} games, oldest first">
                        {#each form as m (m.matchId)}
                            <span
                                class="h-2 flex-1 rounded-full {m.outcome === 'win' ? 'bg-primary' : 'bg-destructive'}"
                                title={m.outcome === "win" ? "Win" : "Loss"}
                            ></span>
                        {/each}
                    </div>
                </Card>
                <Card>
                    <p class="text-sm text-muted-foreground">Streak</p>
                    <p class="mt-1 font-heading text-3xl font-semibold">
                        {run.current ? `${run.current.length} ${run.current.kind === "win" ? "wins" : "losses"}` : "-"}
                    </p>
                    <p class="text-sm text-muted-foreground">
                        Best {run.longestWin} wins, worst {run.longestLoss} losses
                    </p>
                </Card>
                <Card>
                    <p class="text-sm text-muted-foreground">Playtime</p>
                    <p class="mt-1 font-heading text-3xl font-semibold">{formatPlaytime(totalPlaytime(windowed))}</p>
                    <p class="text-sm text-muted-foreground">{windowed.length} matches</p>
                </Card>
            </div>

            {#if mostPlayed}
                <div class="grid gap-3 sm:grid-cols-2">
                    <Card class="flex items-center gap-4">
                        {@render heroIcon(mostPlayed.heroId, "size-14")}
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
                        {#if bestHero}
                            {@render heroIcon(bestHero.heroId, "size-14")}
                            <div class="min-w-0">
                                <p class="text-sm text-muted-foreground">Best winrate (5+ games)</p>
                                <p class="truncate font-heading text-xl">
                                    {heroes[bestHero.heroId]?.name ?? `Hero ${bestHero.heroId}`}
                                </p>
                                <p class="text-sm text-muted-foreground">
                                    {pct(bestHero.winrate)} over {bestHero.games} games
                                </p>
                            </div>
                        {:else}
                            <p class="text-sm text-muted-foreground">
                                Play 5 games on a hero to see your best winrate.
                            </p>
                        {/if}
                    </Card>
                </div>
            {/if}
        {/if}

        <Card as="section">
            <div class="flex flex-wrap items-baseline justify-between gap-2">
                <h2 class="text-xl">Region leaderboard</h2>
                <span class="text-sm {board.regionReady ? 'text-primary' : 'text-muted-foreground'}">
                    {board.regionReady ? "Eligible" : "Not yet eligible"}
                </span>
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
                Approximate. Counts your ranked and unranked games known to the API. Valve does not publish which queues
                count.
            </p>
            <div class="mt-3 grid gap-4 sm:grid-cols-2">
                {@render bar(
                    `Games in the last ${regionRule.windowDays} days`,
                    board.recentGames,
                    regionRule.gamesInWindow,
                )}
                {@render bar("Total games on your account", board.totalGames, regionRule.totalGames)}
            </div>
        </Card>

        <section>
            <h2 class="text-xl">Heroes</h2>
            <p class="mb-3 mt-1 text-sm text-muted-foreground">
                Stats follow the filters above. Hero leaderboard progress is always the last {heroRule.windowDays} days plus
                lifetime wins, and also needs {heroRule.totalGames} total games on your account.
            </p>
            <ul class="flex flex-col gap-3">
                {#each heroRows as row (row.heroId)}
                    {@const hero = heroes[row.heroId]}
                    {@const st = row.stat}
                    <Card as="li">
                        <div class="flex items-center gap-4">
                            {@render heroIcon(row.heroId, "size-16")}
                            <div class="min-w-0 flex-1">
                                <div class="flex items-baseline justify-between gap-3">
                                    <p class="truncate font-heading text-xl">{hero?.name ?? `Hero ${row.heroId}`}</p>
                                    <p
                                        class="shrink-0 text-sm {row.board.ready
                                            ? 'text-primary'
                                            : 'text-muted-foreground'}"
                                    >
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
                            {@render bar(
                                `Games, last ${heroRule.windowDays} days`,
                                row.board.recentGames,
                                heroRule.gamesInWindow,
                            )}
                            {@render bar("Lifetime wins", row.board.wins, heroRule.lifetimeWins)}
                        </div>
                    </Card>
                {/each}
            </ul>
        </section>
    {/if}
</Page>
