<script lang="ts">
    import { onMount } from "svelte";
    import { ArrowRight, ChartLine, Newspaper, Timer, Trophy, User, Wrench } from "@lucide/svelte";
    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";

    import { FEATURES } from "$lib/features/registry";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { greeting, relativeDay, sessionSeed } from "$lib/features/home/home";
    import { alerts } from "$lib/features/alerts/alerts.svelte";
    import { formatPublished } from "$lib/features/alerts/alerts";
    import { stats } from "$lib/features/stats/stats.svelte";
    import { formatPlaytime } from "$lib/features/stats/stats";
    import { rankTrack, standing, subrankAt, windowStats } from "$lib/features/stats/rank";
    import { groupSessions, sessionVerdict, summarizeSession, type Verdict } from "$lib/features/stats/sessions";
    import { firewallCapability, getGameDefinitions, listBlockedGroupIds } from "$lib/features/server-picker/api";
    import { readCachedServerData } from "$lib/features/server-picker/cache";
    import { readVoiceBan } from "$lib/features/voice-bans/api";
    import { parseVoiceBan } from "$lib/features/voice-bans/voice-ban";
    import { listDemos } from "$lib/features/demos/api";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";
    import { countdown, nextMaintenance, WEEKDAYS } from "$lib/features/settings/maintenance";

    interface Summary {
        blocked: number | null;
        mutes: number | null;
        replays: number | null;
    }

    let summary = $state<Summary>({ blocked: null, mutes: null, replays: null });
    let nextMaintenanceAt = $state<number | null>(null);

    const maintenanceLine = $derived.by(() => {
        if (nextMaintenanceAt === null) return "";
        const when = new Date(nextMaintenanceAt * 1000);
        const day = WEEKDAYS[when.getDay() === 0 ? 6 : when.getDay() - 1];
        const time = when.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        return `Usually starts ${day} at ${time}, in ${countdown(nextMaintenanceAt - Date.now() / 1000)}.`;
    });

    const account = $derived(steamAccount.account);
    const name = $derived(account?.personaName || null);

    async function loadBlocked(): Promise<number | null> {
        const capability = await firewallCapability();
        if (!capability.supported) return null;
        const game = (await getGameDefinitions())[0];
        if (!game) return null;
        const cached = await readCachedServerData(game.id);
        if (!cached) return null;
        const ids = [...new Set([...cached.clustered, ...cached.unclustered].map((g) => g.id))];
        return (await listBlockedGroupIds(ids)).length;
    }

    onMount(() => {
        void loadBlocked().then(
            (n) => (summary.blocked = n),
            () => {},
        );
        void readVoiceBan().then(
            (file) => {
                summary.mutes = parseVoiceBan(file.text).users.length;
            },
            () => {},
        );
        void settings.ready
            .then(() => nextMaintenance())
            .then(
                (t) => (nextMaintenanceAt = t),
                () => {},
            );
        void listDemos().then(
            (l) => (summary.replays = l.demos.length),
            () => {},
        );
    });

    const FORM_WINDOW = 20;
    const VERDICT_LABEL: Record<Verdict, string> = {
        excellent: "Excellent session",
        good: "Good session",
        bad: "Rough session",
        neutral: "Even session",
    };

    const accountId = $derived(account?.steamId32 ?? null);
    const statsReady = $derived(stats.status === "ready");
    const track = $derived(rankTrack(stats.matches));
    const info = $derived(stats.rankInfo);
    const now = $derived(info ? standing(info) : null);
    const tierName = (tier: number) => stats.ranks.find((t) => t.tier === tier)?.name ?? `Tier ${tier}`;
    const rankLabel = $derived(now ? `${tierName(now.tier)} ${now.sub}` : null);
    const nextRankLabel = $derived.by(() => {
        if (!info) return null;
        const cur = subrankAt(info.finalFlat);
        const next = subrankAt(cur.start + cur.span);
        return `${tierName(next.tier)} ${next.sub}`;
    });
    const rankPercent = $derived(now && now.within !== null ? Math.round((now.within / now.span) * 100) : null);
    const form = $derived(windowStats(track, FORM_WINDOW));
    const lastSession = $derived(groupSessions(stats.matches).at(-1));
    const lastSummary = $derived(lastSession ? summarizeSession(lastSession) : null);
    const latestAlert = $derived(alerts.items[0] ?? null);

    const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
    const pct = (v: number | null) => (v === null ? "-" : `${Math.round(v * 100)}%`);

    $effect(() => {
        if (accountId !== null) void stats.load(accountId);
    });

    const tiles = $derived([
        {
            href: "/server-picker",
            label: "Blocked regions",
            value: summary.blocked === null ? "–" : String(summary.blocked),
        },
        {
            href: "/voice-bans",
            label: "Muted players",
            value: summary.mutes === null ? "–" : String(summary.mutes),
        },
        {
            href: "/demos",
            label: "Saved replays",
            value: summary.replays === null ? "–" : String(summary.replays),
        },
    ]);
</script>

<div class="flex min-h-full items-center justify-center px-8 py-10">
    <div class="flex w-full max-w-6xl flex-col gap-10">
        <header class="flex flex-col items-center gap-4 text-center">
            {#if account?.avatarDataUrl}
                <img src={account.avatarDataUrl} alt="" class="size-28 shrink-0 rounded-xl border border-border" />
            {:else}
                <div class="flex size-28 shrink-0 items-center justify-center rounded-xl border border-border bg-card">
                    <User class="size-14 text-muted-foreground" />
                </div>
            {/if}
            <div class="min-w-0 max-w-full">
                <h1 class="text-4xl leading-tight lg:text-5xl">
                    {greeting(new Date().getHours(), name, sessionSeed)}
                </h1>
            </div>
        </header>

        <section class="grid grid-cols-1 gap-3 sm:grid-cols-3" aria-label="At a glance">
            {#each tiles as tile (tile.href)}
                <a
                    href={tile.href}
                    class="rounded-lg border border-border bg-card px-5 py-4 text-center transition-colors hover:bg-accent/50"
                >
                    <p class="font-heading text-4xl font-semibold tabular-nums text-brass">{tile.value}</p>
                    <p class="mt-1 text-sm text-muted-foreground">{tile.label}</p>
                </a>
            {/each}
        </section>

        {#if maintenanceLine}
            <Button
                type="button"
                variant="unstyled"
                onclick={() => settingsUi.show("notifications")}
                class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3 hover:bg-accent/50"
            >
                <Wrench class="size-4 shrink-0 text-muted-foreground" />
                <div class="min-w-0">
                    <p class="font-heading text-sm font-semibold tracking-wide">Steam maintenance</p>
                    <p class="text-xs text-muted-foreground">
                        {maintenanceLine} Valve publishes no schedule, so this is an estimate.
                        {settings.maintenance.enabled ? "The reminder is on." : "The reminder is off."}
                    </p>
                </div>
            </Button>
        {/if}

        <section class="flex flex-col gap-3" aria-label="Your Deadlock">
            <h2 class="font-heading text-sm font-semibold tracking-wide text-muted-foreground">Your Deadlock</h2>
            <ul class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                <li>
                    <a
                        href="/rank"
                        class="flex h-full flex-col gap-2 rounded-lg border border-border bg-card px-4 py-4 transition-colors hover:bg-accent/50"
                    >
                        <div class="flex items-center gap-2">
                            <Trophy class="size-4 shrink-0 text-brass" />
                            <p class="font-heading text-sm font-semibold tracking-wide">Rank</p>
                        </div>
                        {#if rankLabel}
                            <p class="font-heading text-2xl font-semibold text-brass">{rankLabel}</p>
                            {#if rankPercent !== null}
                                <div class="h-2 overflow-hidden rounded-full bg-muted" role="presentation">
                                    <div class="h-full rounded-full bg-primary" style="width: {rankPercent}%"></div>
                                </div>
                                <p class="text-xs text-muted-foreground">
                                    {rankPercent}% of the way to {nextRankLabel}
                                </p>
                            {/if}
                        {:else}
                            <p class="text-sm text-muted-foreground">
                                {statsReady
                                    ? "No ranked rank yet."
                                    : stats.status === "error"
                                      ? "Could not load."
                                      : "Loading..."}
                            </p>
                        {/if}
                    </a>
                </li>

                <li>
                    <a
                        href="/rank"
                        class="flex h-full flex-col gap-2 rounded-lg border border-border bg-card px-4 py-4 transition-colors hover:bg-accent/50"
                    >
                        <div class="flex items-center gap-2">
                            <ChartLine class="size-4 shrink-0 text-brass" />
                            <p class="font-heading text-sm font-semibold tracking-wide">Recent form</p>
                        </div>
                        {#if form.games > 0}
                            <p class="font-heading text-2xl font-semibold tabular-nums">{pct(form.winrate)}</p>
                            <p class="text-xs text-muted-foreground">
                                {form.wins}W {form.losses}L, {signed(form.net)} rank points, last {form.games} ranked
                            </p>
                        {:else}
                            <p class="text-sm text-muted-foreground">
                                {statsReady
                                    ? "No ranked matches yet."
                                    : stats.status === "error"
                                      ? "Could not load."
                                      : "Loading..."}
                            </p>
                        {/if}
                    </a>
                </li>

                <li>
                    <a
                        href="/alerts"
                        class="flex h-full flex-col gap-2 rounded-lg border border-border bg-card px-4 py-4 transition-colors hover:bg-accent/50"
                    >
                        <div class="flex items-center gap-2">
                            <Newspaper class="size-4 shrink-0 text-brass" />
                            <p class="font-heading text-sm font-semibold tracking-wide">Latest update</p>
                            {#if alerts.unread > 0}
                                <Badge class="ml-auto">{alerts.unread} new</Badge>
                            {/if}
                        </div>
                        {#if latestAlert}
                            <p class="line-clamp-2 text-sm font-medium">{latestAlert.title}</p>
                            <p class="text-xs text-muted-foreground">{formatPublished(latestAlert.published)}</p>
                        {:else}
                            <p class="text-sm text-muted-foreground">Nothing yet. Open Updates to check.</p>
                        {/if}
                    </a>
                </li>

                <li>
                    <a
                        href="/sessions"
                        class="flex h-full flex-col gap-2 rounded-lg border border-border bg-card px-4 py-4 transition-colors hover:bg-accent/50"
                    >
                        <div class="flex items-center gap-2">
                            <Timer class="size-4 shrink-0 text-brass" />
                            <p class="font-heading text-sm font-semibold tracking-wide">Last session</p>
                        </div>
                        {#if lastSummary}
                            <p class="font-heading text-2xl font-semibold">
                                {relativeDay(lastSummary.startTime, Date.now() / 1000)}
                            </p>
                            <p class="text-xs text-muted-foreground">
                                {VERDICT_LABEL[sessionVerdict(lastSummary)]}: {lastSummary.wins}W {lastSummary.losses}L
                                in {formatPlaytime(lastSummary.durationS)}{lastSummary.netDelta === null
                                    ? ""
                                    : `, ${signed(lastSummary.netDelta)}`}
                            </p>
                        {:else}
                            <p class="text-sm text-muted-foreground">
                                {statsReady
                                    ? "No matches yet."
                                    : stats.status === "error"
                                      ? "Could not load."
                                      : "Loading..."}
                            </p>
                        {/if}
                    </a>
                </li>
            </ul>
        </section>

        <section class="flex flex-col gap-3" aria-label="Tools">
            <h2 class="font-heading text-sm font-semibold tracking-wide text-muted-foreground">Tools</h2>
            <ul class="flex flex-wrap justify-center gap-3">
                {#each FEATURES as feature (feature.id)}
                    {@const Icon = feature.icon}
                    <li class="w-full md:w-[calc((100%-0.75rem)/2)] lg:w-[calc((100%-1.5rem)/3)]">
                        <a
                            href={feature.href}
                            class="group flex h-full items-center gap-3 rounded-lg border border-border bg-card px-4 py-3 transition-colors hover:bg-accent/50"
                        >
                            <Icon class="size-5 shrink-0 text-brass" />
                            <div class="min-w-0 flex-1">
                                <p class="font-heading text-sm font-semibold tracking-wide">{feature.label}</p>
                                <p class="mt-0.5 text-xs text-muted-foreground">{feature.description}</p>
                            </div>
                            <ArrowRight
                                class="size-4 shrink-0 text-muted-foreground/60 transition-transform group-hover:translate-x-0.5"
                            />
                        </a>
                    </li>
                {/each}
            </ul>
        </section>
    </div>
</div>
