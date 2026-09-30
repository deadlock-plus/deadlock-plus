<script lang="ts">
    import { onMount } from "svelte";
    import { ArrowDown, ArrowUp, CircleHelp, LoaderCircle, RefreshCw, Shield, ShieldOff } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
    import { breakEvenWinrate, climbForecast, type Eta } from "$lib/features/stats/climb";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { stats } from "$lib/features/stats/stats.svelte";
    import CachedNote from "$lib/features/stats/components/cached-note.svelte";
    import {
        badgeParts,
        progressSeries,
        rankChanges,
        rankTrack,
        standing,
        subrankAt,
        gainForWin,
        lossOutcome,
        windowStats,
        winsToNext,
        winStreak,
    } from "$lib/features/stats/rank";

    const WINDOWS = [20, 50, 100];
    const FORM_WINDOW = 20;
    const LIST_ROWS = 8;

    let heroes = $state<Record<number, Hero>>({});
    let shown = $state(50);

    const accountId = $derived(steamAccount.account?.steamId32 ?? null);
    const track = $derived(rankTrack(stats.matches));
    const info = $derived(stats.rankInfo);
    const now = $derived(info ? standing(info) : null);
    const series = $derived(info ? progressSeries(track, info) : []);
    const form = $derived(windowStats(track, FORM_WINDOW));
    const changes = $derived(rankChanges(track).slice(0, LIST_ROWS));
    const recent = $derived(track.slice(-LIST_ROWS).reverse());
    const topTier = $derived(Math.max(0, ...stats.ranks.map((t) => t.tier)));
    const atTop = $derived(now !== null && now.tier >= topTier);
    const streak = $derived(winStreak(track));
    const toNext = $derived(now && !atTop && now.within !== null ? winsToNext(now.within, now.span, streak) : null);
    const nextLoss = $derived(
        now && now.within !== null && info ? lossOutcome(now.within, info.shieldsLeft ?? 0) : null,
    );

    const breakEven = breakEvenWinrate();
    const forecast = $derived(info && !atTop ? climbForecast(track, info.finalFlat, Date.now() / 1000) : null);

    const etaText = (e: Eta) => {
        if (e.daysLow === null) return "Not climbing at your recent pace";
        if (e.daysHigh === null) return `${e.daysLow} days or more`;
        return e.daysLow === e.daysHigh
            ? `About ${e.daysLow} ${e.daysLow === 1 ? "day" : "days"}`
            : `${e.daysLow} to ${e.daysHigh} days`;
    };

    const tierInfo = (tier: number) => stats.ranks.find((t) => t.tier === tier);
    const nameOf = (badge: number) => {
        const p = badgeParts(badge);
        return p ? `${tierInfo(p.tier)?.name ?? `Tier ${p.tier}`} ${p.sub}` : "-";
    };
    const partsName = (p: { tier: number; sub: number }) => `${tierInfo(p.tier)?.name ?? `Tier ${p.tier}`} ${p.sub}`;
    const nextName = $derived.by(() => {
        const cur = subrankAt(info?.finalFlat ?? 0);
        return partsName(subrankAt(cur.start + cur.span));
    });

    const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
    const pct = (v: number | null) => (v === null ? "-" : `${Math.round(v * 100)}%`);
    const day = (s: number) => new Date(s * 1000).toLocaleDateString(undefined, { day: "numeric", month: "short" });

    const W = 760;
    const H = 280;
    const PAD = { l: 96, r: 16, t: 14, b: 14 };

    const chart = $derived.by(() => {
        const pts = series.slice(-shown);
        if (!info || pts.length < 2) return null;
        const flats = pts.map((p) => p.flat);
        const lowest = subrankAt(Math.min(...flats));
        const highest = subrankAt(Math.max(...flats));
        const lo = lowest.start;
        const hi = highest.start + highest.span;
        const y = (v: number) => PAD.t + (1 - (v - lo) / (hi - lo)) * (H - PAD.t - PAD.b);
        const x = (i: number) => PAD.l + (i / (pts.length - 1)) * (W - PAD.l - PAD.r);
        const edges: number[] = [];
        for (let v = lo; v <= hi; v = subrankAt(v).start + subrankAt(v).span) edges.push(v);
        const every = Math.ceil(edges.length / 7);
        const lines = edges.map((v) => ({ y: y(v), label: partsName(subrankAt(v)) })).filter((_, i) => i % every === 0);
        return {
            lines,
            d: pts.map((p, i) => `${i ? "L" : "M"}${x(i).toFixed(1)} ${y(p.flat).toFixed(1)}`).join(" "),
            dots: pts.map((p, i) => ({ cx: x(i), cy: y(p.flat), p })),
            from: day(pts[0].startTime),
            to: day(pts.at(-1)!.startTime),
        };
    });

    $effect(() => {
        if (accountId !== null) void stats.load(accountId);
    });

    onMount(() => {
        void loadHeroes().then((h) => (heroes = h));
    });
</script>

<Page size="lg">
    <PageHeader
        size="lg"
        title="Rank"
        subtitle="Where you stand in ranked, and how you got there. From the Deadlock API."
    >
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
    {:else if !info || !now || track.length === 0}
        <EmptyState size="base" spacing="xl"
            >No rank to show yet. It appears once the API has a ranked match of yours past placement games.</EmptyState
        >
    {:else}
        <CachedNote />
        {@const tier = tierInfo(now.tier)}
        <Card as="section" padding="lg" class="grid gap-4 md:grid-cols-[1fr_auto]">
            <div class="flex items-center gap-5">
                {#if tier?.image}
                    <img src={tier.image} alt="" class="size-28 shrink-0 object-contain" />
                {/if}
                <div class="min-w-0 flex-1">
                    <p class="text-sm text-muted-foreground">Current rank</p>
                    <p class="font-heading text-4xl font-semibold" style:color={tier?.color}>{partsName(now)}</p>
                    {#if atTop}
                        <p class="mt-2 text-sm text-muted-foreground">
                            Top tier. Subranks here are percentile cuts, so there is no progress bar.
                        </p>
                    {:else}
                        {#if now.within !== null}
                            <div
                                class="mt-3 h-2.5 w-full overflow-hidden rounded-full bg-muted"
                                role="progressbar"
                                aria-valuemin="0"
                                aria-valuemax={now.span}
                                aria-valuenow={now.within ?? 0}
                            >
                                <div
                                    class="h-full rounded-full bg-primary"
                                    style:width="{(now.within / now.span) * 100}%"
                                ></div>
                            </div>
                            <p class="mt-1.5 text-sm text-muted-foreground">
                                {now.within} / {now.span} to {nextName}{toNext !== null
                                    ? `. ${toNext} ${toNext === 1 ? "win" : "wins"} in a row from here.`
                                    : "."}
                            </p>
                        {:else}
                            <p class="mt-2 text-sm text-muted-foreground">
                                Your progress just crossed a boundary and the API still shows the old badge, so there is
                                no bar until the next match.
                            </p>
                        {/if}
                    {/if}
                    {#if (info.placementLeft ?? 0) > 0}
                        <p class="mt-1 text-sm text-muted-foreground">{info.placementLeft} placement games left.</p>
                    {/if}
                </div>
            </div>

            <div
                class="flex flex-col items-center justify-center gap-2 rounded-md border border-border bg-background/40 px-6 py-4 md:min-w-52"
            >
                <p class="text-sm text-muted-foreground">Demotion shields</p>
                {#if info.shieldsLeft === null}
                    <p class="text-sm text-muted-foreground">Not reported</p>
                {:else if info.shieldsLeft === 0}
                    <ShieldOff class="size-9 text-destructive" aria-hidden="true" />
                    <p class="font-heading text-lg">None left</p>
                    <p class="text-center text-xs text-muted-foreground">A loss can take you down a rank.</p>
                {:else}
                    <div class="flex gap-1.5" aria-hidden="true">
                        {#each { length: info.shieldsLeft } as _, i (i)}
                            <Shield class="size-9 fill-primary/25 text-primary" />
                        {/each}
                    </div>
                    <p class="font-heading text-lg">
                        {info.shieldsLeft}
                        {info.shieldsLeft === 1 ? "shield" : "shields"} left
                    </p>
                    <p class="text-center text-xs text-muted-foreground">
                        Each one absorbs a loss that would demote you.
                    </p>
                {/if}
            </div>
        </Card>

        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <Card>
                <p class="text-sm text-muted-foreground">Last {FORM_WINDOW} ranked</p>
                <p class="mt-1 font-heading text-3xl font-semibold">{pct(form.winrate)}</p>
                <p class="text-sm text-muted-foreground">
                    {form.wins}W {form.losses}L{form.games < Math.min(FORM_WINDOW, track.length)
                        ? `, ${Math.min(FORM_WINDOW, track.length) - form.games} not scored`
                        : ""}
                </p>
            </Card>
            <Card>
                <p class="text-sm text-muted-foreground">Progress, same games</p>
                <p class="mt-1 font-heading text-3xl font-semibold">{signed(form.net)}</p>
                <p class="text-sm text-muted-foreground">1000 points is a subrank</p>
            </Card>
            <Card>
                <p class="text-sm text-muted-foreground">Next win</p>
                <p class="mt-1 font-heading text-3xl font-semibold">+{gainForWin(streak + 1)}</p>
                <p class="text-sm text-muted-foreground">
                    {streak === 0 ? "No win streak" : `${streak} ${streak === 1 ? "win" : "wins"} in a row`}
                </p>
            </Card>
            <Card>
                <p class="text-sm text-muted-foreground">Next loss</p>
                {#if nextLoss}
                    <p class="mt-1 font-heading text-3xl font-semibold">-{nextLoss.lost}</p>
                    <p class="text-sm text-muted-foreground">
                        {#if nextLoss.usesShield}
                            {nextLoss.lost === 0 ? "Only a shield" : "Plus a shield"}
                        {:else if nextLoss.demotes}
                            No shield: drops a subrank
                        {:else}
                            No shield used
                        {/if}
                    </p>
                {:else}
                    <p class="mt-1 font-heading text-3xl font-semibold">-</p>
                {/if}
            </Card>
        </div>

        <div class="grid gap-3 md:grid-cols-2">
            <Card>
                <p class="text-sm text-muted-foreground">Winrate to hold your rank</p>
                <p class="mt-1 font-heading text-3xl font-semibold">{Math.round(breakEven * 100)}%</p>
                <p class="text-sm text-muted-foreground">
                    {#if form.winrate === null}
                        Win streaks pay more than losses cost, so you don't need 50%.
                    {:else}
                        You are at {pct(form.winrate)} over your last {FORM_WINDOW}: {form.winrate >= breakEven
                            ? "climbing"
                            : "sliding"}.
                    {/if}
                </p>
            </Card>
            <Card>
                <p class="text-sm text-muted-foreground">Climb forecast</p>
                {#if forecast}
                    <p class="mt-1 font-heading text-3xl font-semibold">{signed(Math.round(forecast.perDay))} a day</p>
                    <p class="text-sm text-muted-foreground">
                        {nextName}: {etaText(forecast.subrank)}{forecast.tier
                            ? `. Next tier: ${etaText(forecast.tier)}`
                            : ""}.
                    </p>
                {:else}
                    <p class="mt-1 font-heading text-3xl font-semibold">-</p>
                    <p class="text-sm text-muted-foreground">Needs about 8 ranked games in the last two weeks.</p>
                {/if}
            </Card>
        </div>

        <Card as="section">
            <div class="mb-2 flex flex-wrap items-center justify-between gap-2">
                <h2 class="text-xl">Progress</h2>
                <div class="flex gap-2">
                    {#each WINDOWS as n (n)}
                        <Button size="sm" variant={shown === n ? "default" : "outline"} onclick={() => (shown = n)}
                            >Last {n}</Button
                        >
                    {/each}
                </div>
            </div>
            {#if chart}
                <svg
                    viewBox="0 0 {W} {H}"
                    class="h-auto w-full"
                    role="img"
                    aria-label="Rank progress over your recent ranked matches"
                >
                    {#each chart.lines as l (l.y)}
                        <line
                            x1={PAD.l}
                            x2={W - PAD.r}
                            y1={l.y}
                            y2={l.y}
                            stroke="var(--color-border)"
                            stroke-width="1"
                        />
                        <text
                            x={PAD.l - 8}
                            y={l.y + 4}
                            text-anchor="end"
                            font-size="12"
                            fill="var(--color-muted-foreground)">{l.label}</text
                        >
                    {/each}
                    <path
                        d={chart.d}
                        fill="none"
                        stroke="var(--color-primary)"
                        stroke-width="1.5"
                        stroke-opacity="0.6"
                    />
                    {#each chart.dots as d (d.p.matchId)}
                        {#if d.p.demotionProtected}
                            <circle
                                cx={d.cx}
                                cy={d.cy}
                                r="7"
                                fill="none"
                                stroke="var(--color-primary)"
                                stroke-width="1.5"
                            />
                        {/if}
                        <circle
                            cx={d.cx}
                            cy={d.cy}
                            r="3"
                            fill={d.p.outcome === "win"
                                ? "var(--color-primary)"
                                : d.p.outcome === "loss"
                                  ? "var(--color-destructive)"
                                  : "var(--color-muted-foreground)"}
                        />
                    {/each}
                </svg>
                <p class="mt-1 flex flex-wrap gap-x-4 text-xs text-muted-foreground">
                    <span>{chart.from} to {chart.to}</span>
                    <span>Filled dots: green win, red loss. Ringed dot: a shield absorbed the loss.</span>
                </p>
            {:else}
                <p class="text-sm text-muted-foreground">Needs at least two ranked matches to draw a line.</p>
            {/if}
        </Card>

        <div class="grid gap-5 md:grid-cols-2">
            <section>
                <h2 class="mb-2 text-xl">Recent ranked matches</h2>
                <ul class="flex flex-col gap-2">
                    {#each recent as p (p.matchId)}
                        {@const hero = heroes[p.heroId]}
                        <Card as="li" radius="md" padding="none" class="flex h-14 items-center gap-3 px-3">
                            <div
                                class="flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted"
                            >
                                {#if hero?.icon}
                                    <img src={hero.icon} alt="" class="size-full object-cover" />
                                {:else}
                                    <CircleHelp class="size-4 text-muted-foreground" aria-label="Unknown hero" />
                                {/if}
                            </div>
                            <div class="min-w-0 flex-1">
                                <p class="truncate text-sm font-medium">{hero?.name ?? `Hero ${p.heroId}`}</p>
                                <p class="text-xs text-muted-foreground">{day(p.startTime)}</p>
                            </div>
                            {#if p.demotionProtected}
                                <Shield
                                    class="size-4 fill-primary/25 text-primary"
                                    aria-label="A shield absorbed this loss"
                                />
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
                            <span class="w-14 text-right text-sm tabular-nums"
                                >{p.delta === null ? "-" : signed(p.delta)}</span
                            >
                        </Card>
                    {/each}
                </ul>
            </section>

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
                                    <p class="truncate text-sm font-medium">{nameOf(c.to)}</p>
                                    <p class="text-xs text-muted-foreground">
                                        {c.promoted ? "Promoted from" : "Demoted from"}
                                        {nameOf(c.from)} · {day(c.startTime)}
                                    </p>
                                </div>
                            </Card>
                        {/each}
                    </ul>
                {/if}
            </section>
        </div>
    {/if}
</Page>
