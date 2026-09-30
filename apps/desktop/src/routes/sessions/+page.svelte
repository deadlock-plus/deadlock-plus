<script lang="ts">
    import { LoaderCircle, RefreshCw, Trophy } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { stats } from "$lib/features/stats/stats.svelte";
    import CachedNote from "$lib/features/stats/components/cached-note.svelte";
    import { filterScope, formatPlaytime, type Scope } from "$lib/features/stats/stats";
    import {
        groupSessions,
        shouldSuggestBreak,
        sessionHighlight,
        sessionVerdict,
        summarizeSession,
        type Verdict,
        type Session,
    } from "$lib/features/stats/sessions";
    import { buildFindings } from "$lib/features/stats/insights";
    import Findings from "$lib/features/stats/components/findings.svelte";

    const SCOPES: { id: Scope; label: string }[] = [
        { id: "ranked", label: "Ranked" },
        { id: "unranked", label: "Unranked" },
        { id: "all", label: "All modes" },
    ];
    const LOSS_STREAK = 3;
    const SHOWN = 15;

    let scope = $state<Scope>("ranked");

    const accountId = $derived(steamAccount.account?.steamId32 ?? null);
    const scoped = $derived(filterScope(stats.matches, scope));
    const sessions = $derived(groupSessions(scoped));
    const recent = $derived(sessions.slice(-SHOWN).reverse());
    const insights = $derived(buildFindings(sessions, -new Date().getTimezoneOffset()));
    const suggestBreak = $derived(settings.breakHint && shouldSuggestBreak(sessions, LOSS_STREAK, Date.now() / 1000));

    const highlight = $derived(sessionHighlight(sessions, Date.now() / 1000));
    const highlightTitle = $derived.by(() => {
        if (!highlight) return "";
        if (highlight.kind === "best") return "Your best recent session";
        return highlight.verdict === "excellent"
            ? "Excellent last session. Well played."
            : "Nice work. Your last session went well.";
    });

    const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
    const when = (s: number) =>
        new Date(s * 1000).toLocaleString(undefined, {
            weekday: "short",
            day: "numeric",
            month: "short",
            hour: "2-digit",
            minute: "2-digit",
        });
    const EDGE: Record<Verdict, string> = {
        excellent: "border-l-brass",
        good: "border-l-primary",
        bad: "border-l-destructive",
        neutral: "border-l-muted-foreground/60",
    };
    const VERDICT_LABEL: Record<Verdict, string> = {
        excellent: "Excellent session",
        good: "Good session",
        bad: "Rough session",
        neutral: "Even session",
    };

    $effect(() => {
        if (accountId !== null) void stats.load(accountId);
    });
</script>

{#snippet sessionRow(s: Session)}
    {@const sum = summarizeSession(s)}
    {@const verdict = sessionVerdict(sum)}
    <li
        class="flex flex-wrap items-center gap-x-6 gap-y-2 rounded-md border border-l-4 border-border bg-card px-5 py-4 {EDGE[
            verdict
        ]}"
    >
        <div class="min-w-40 flex-1">
            <p class="text-base font-medium">{when(sum.startTime)}</p>
            <p class="text-sm text-muted-foreground">{VERDICT_LABEL[verdict]}</p>
        </div>
        <dl class="flex gap-8 text-right text-sm">
            <div>
                <dt class="text-xs text-muted-foreground">Games</dt>
                <dd class="text-base">{sum.games}</dd>
            </div>
            <div>
                <dt class="text-xs text-muted-foreground">W / L</dt>
                <dd class="text-base">{sum.wins} / {sum.losses}</dd>
            </div>
            <div>
                <dt class="text-xs text-muted-foreground">Length</dt>
                <dd class="text-base">{formatPlaytime(sum.durationS)}</dd>
            </div>
            <div>
                <dt class="text-xs text-muted-foreground">Rank change</dt>
                <dd class="text-base">{sum.netDelta === null ? "-" : signed(sum.netDelta)}</dd>
            </div>
        </dl>
    </li>
{/snippet}

<div class="mx-auto flex min-h-full max-w-5xl flex-col gap-5 px-8 pb-12 pt-8">
    <header class="flex flex-wrap items-start justify-between gap-3">
        <div>
            <h1 class="text-3xl">Sessions</h1>
            <p class="mt-1 text-base text-muted-foreground">
                Matches grouped into sessions. A new session starts after 90 minutes without playing.
            </p>
        </div>
        <Button
            variant="outline"
            size="sm"
            disabled={accountId === null || stats.status === "loading"}
            onclick={() => accountId !== null && stats.load(accountId, true)}
        >
            <RefreshCw class={stats.status === "loading" ? "animate-spin" : ""} /> Refresh
        </Button>
    </header>

    {#if steamAccount.loaded && accountId === null}
        <p class="py-16 text-center text-base text-muted-foreground">
            No Steam account found, so there is no history to load.
        </p>
    {:else if stats.status === "error"}
        <p class="py-16 text-center text-base text-destructive">Could not load your match history. {stats.error}</p>
    {:else if stats.status !== "ready"}
        <p class="flex items-center justify-center gap-2 py-16 text-base text-muted-foreground">
            <LoaderCircle class="size-4 animate-spin" /> Loading your match history...
        </p>
    {:else}
        <CachedNote />
        <div class="flex flex-wrap items-center gap-2">
            {#each SCOPES as s (s.id)}
                <Button size="sm" variant={scope === s.id ? "default" : "outline"} onclick={() => (scope = s.id)}
                    >{s.label}</Button
                >
            {/each}
        </div>

        <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center justify-between gap-4">
                <div class="flex flex-col gap-1">
                    <label for="break-hint" class="font-heading text-sm font-semibold tracking-wide"
                        >Break reminder</label
                    >
                    <p class="text-sm text-muted-foreground">
                        Shows a note here when your current session has {LOSS_STREAK} losses in a row. Off by default.
                    </p>
                </div>
                <Switch
                    id="break-hint"
                    checked={settings.breakHint}
                    onCheckedChange={(v) => settings.setBreakHint(v)}
                />
            </div>
            {#if suggestBreak}
                <p class="mt-3 rounded-md border border-border bg-muted px-3 py-2 text-sm">
                    {LOSS_STREAK} losses in a row this session. A short break might be worth it. Up to you.
                </p>
            {/if}
        </section>

        {#if sessions.length === 0}
            <p class="py-12 text-center text-base text-muted-foreground">No matches in this selection.</p>
        {:else}
            {#if highlight}
                <section>
                    <h2 class="mb-2 flex items-center gap-2 text-xl">
                        <Trophy class="size-5 text-brass" aria-hidden="true" />
                        {highlightTitle}
                    </h2>
                    <ul>{@render sessionRow(highlight.session)}</ul>
                </section>
            {/if}

            <section>
                <h2 class="mb-2 text-xl">Latest sessions</h2>
                <ul class="flex flex-col gap-2">
                    {#each recent as s (s.matches[0].matchId)}
                        {@render sessionRow(s)}
                    {/each}
                </ul>
            </section>

            <section>
                <h2 class="text-xl">What your results say</h2>
                <p class="mb-3 mt-1 text-sm text-muted-foreground">
                    The line on each bar is your overall winrate{insights.baseline.winrate === null
                        ? ""
                        : `, ${Math.round(insights.baseline.winrate * 100)}% over ${insights.baseline.games} games`}. A
                    difference only counts once enough games back it up.
                </p>
                <Findings findings={insights.findings} baseline={insights.baseline.winrate} />
            </section>
        {/if}
    {/if}
</div>
