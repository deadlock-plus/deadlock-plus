<script lang="ts">
    import { ChartLine, ChevronDown, ChevronUp, Coins, Swords } from "@lucide/svelte";
    import { formatNumber, t } from "$lib/core/i18n.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import { formatClock } from "$lib/features/live/live";
    import Button from "$lib/ui/button.svelte";
    import {
        chartDomain,
        linePath,
        playerLine,
        teamLeadLine,
        valueAt,
        xOf,
        yOf,
        type ChartBox,
        type Metric,
    } from "../../../deep-dive/chart";
    import { teamRoster } from "../../../deep-dive/roster";
    import { allPlayers, type MatchDetail, type MatchPlayer } from "../../../detail";
    import HeroChip from "./hero-chip.svelte";
    import SectionCard from "./section-card.svelte";
    import TeamMark from "./team-mark.svelte";

    let { detail, selected, heroes }: { detail: MatchDetail; selected: MatchPlayer; heroes: Record<number, Hero> } =
        $props();

    let metric = $state<Metric>("souls");
    let hoverT = $state<number | null>(null);

    const box: ChartBox = { width: 600, height: 160, padLeft: 4, padRight: 4, padTop: 6, padBottom: 6 };

    const lines = $derived(allPlayers(detail).map((p) => ({ p, line: playerLine(p, metric) })));
    const lead = $derived(teamLeadLine(detail, metric));
    const available = $derived(lead.length > 1);

    const playerDom = $derived(chartDomain(lines.map((l) => l.line)));
    const leadDom = $derived(chartDomain([lead]));
    const paths = $derived(lines.map((l) => ({ slot: l.p.slot, team: l.p.team, d: linePath(l.line, playerDom, box) })));
    const others = $derived(paths.filter((p) => p.slot !== selected.slot));
    const selectedPath = $derived(paths.find((p) => p.slot === selected.slot)?.d ?? "");
    const selectedLine = $derived(lines.find((l) => l.p.slot === selected.slot)?.line ?? []);
    const leadPath = $derived(linePath(lead, leadDom, box));
    const zeroY = $derived(yOf(0, leadDom, box));

    const num = (v: number) => formatNumber(Math.round(v));
    const signed = (v: number) => (v > 0 ? `+${num(v)}` : num(v));
    const clock = (s: number) => formatClock(Math.round(s)) ?? "";
    const roster = $derived(teamRoster(detail));
    const teamColor = (team: string) => (team === "hidden-king" ? "var(--dd-hk)" : "var(--dd-am)");

    function track(e: PointerEvent, tMax: number) {
        const r = (e.currentTarget as SVGElement).getBoundingClientRect();
        if (r.width <= 0) return;
        hoverT = Math.min(Math.max((e.clientX - r.left) / r.width, 0), 1) * tMax;
    }

    const leadReadout = $derived(hoverT === null ? null : valueAt(lead, hoverT));
    const playerReadout = $derived(hoverT === null ? null : valueAt(selectedLine, hoverT));
    const leadingTeam = $derived(
        leadReadout === null || leadReadout === 0 ? null : leadReadout > 0 ? "hidden-king" : "archmother",
    );
</script>

<SectionCard title={t("match_history.deep_dive.progress.heading")} icon={ChartLine} {available}>
    <div class="mb-3 flex flex-wrap items-center gap-2">
        <Button
            size="sm"
            variant={metric === "souls" ? "default" : "outline"}
            aria-pressed={metric === "souls"}
            onclick={() => (metric = "souls")}
        >
            <Coins size={14} aria-hidden="true" />
            {t("match_history.deep_dive.progress.souls")}
        </Button>
        <Button
            size="sm"
            variant={metric === "playerDamage" ? "default" : "outline"}
            aria-pressed={metric === "playerDamage"}
            onclick={() => (metric = "playerDamage")}
        >
            <Swords size={14} aria-hidden="true" />
            {t("match_history.deep_dive.progress.damage")}
        </Button>
    </div>

    <h3 class="mb-1 text-xs font-medium text-muted-foreground">
        {t("match_history.deep_dive.progress.lead_heading")}
    </h3>
    <div class="relative">
        <svg
            viewBox="0 0 {box.width} {box.height}"
            class="h-40 w-full touch-none"
            preserveAspectRatio="none"
            role="img"
            aria-label={t("match_history.deep_dive.progress.lead_aria")}
            onpointermove={(e) => track(e, leadDom.tMax)}
            onpointerleave={() => (hoverT = null)}
        >
            <line
                x1={box.padLeft}
                x2={box.width - box.padRight}
                y1={zeroY}
                y2={zeroY}
                stroke="var(--color-border)"
                stroke-width="1"
                vector-effect="non-scaling-stroke"
            />
            <path
                d={leadPath}
                fill="none"
                stroke="var(--color-foreground)"
                stroke-width="2"
                vector-effect="non-scaling-stroke"
            />
            {#if hoverT !== null}
                <line
                    x1={xOf(hoverT, leadDom, box)}
                    x2={xOf(hoverT, leadDom, box)}
                    y1="0"
                    y2={box.height}
                    stroke="var(--color-muted-foreground)"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            {/if}
        </svg>
        <span class="absolute left-1 top-0 text-[10px] text-muted-foreground">{signed(leadDom.vMax)}</span>
        <span class="absolute bottom-0 left-1 text-[10px] text-muted-foreground">{signed(leadDom.vMin)}</span>
    </div>
    <p class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
        <span class="inline-flex items-center gap-1">
            <ChevronUp size={14} aria-hidden="true" />
            <TeamMark team="hidden-king" />
            <ChevronDown size={14} aria-hidden="true" />
            <TeamMark team="archmother" />
            {t("match_history.deep_dive.progress.lead_hint")}
        </span>
        {#if leadReadout !== null && hoverT !== null}
            <span class="inline-flex items-center gap-1.5 text-foreground">
                {clock(hoverT)} · {signed(leadReadout)}
                {#if leadingTeam}<TeamMark team={leadingTeam} />{/if}
            </span>
        {/if}
    </p>

    <h3 class="mb-1 mt-4 text-xs font-medium text-muted-foreground">
        {t("match_history.deep_dive.progress.player_heading")}
    </h3>
    <div class="relative">
        <svg
            viewBox="0 0 {box.width} {box.height}"
            class="h-40 w-full touch-none"
            preserveAspectRatio="none"
            role="img"
            aria-label={t("match_history.deep_dive.progress.player_aria")}
            onpointermove={(e) => track(e, playerDom.tMax)}
            onpointerleave={() => (hoverT = null)}
        >
            {#each others as o (o.slot)}
                <path
                    d={o.d}
                    fill="none"
                    stroke={teamColor(o.team)}
                    stroke-opacity="0.35"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            {/each}
            <path
                d={selectedPath}
                fill="none"
                stroke={teamColor(selected.team)}
                stroke-width="2.5"
                vector-effect="non-scaling-stroke"
            />
            {#if hoverT !== null}
                <line
                    x1={xOf(hoverT, playerDom, box)}
                    x2={xOf(hoverT, playerDom, box)}
                    y1="0"
                    y2={box.height}
                    stroke="var(--color-muted-foreground)"
                    stroke-width="1"
                    vector-effect="non-scaling-stroke"
                />
            {/if}
        </svg>
        <span class="absolute left-1 top-0 text-[10px] text-muted-foreground">{num(playerDom.vMax)}</span>
        <span class="absolute bottom-0 left-1 text-[10px] text-muted-foreground">{num(playerDom.vMin)}</span>
    </div>
    <div class="mt-2 flex flex-col gap-1.5">
        {#each roster as r (r.team)}
            <div class="flex flex-wrap items-center gap-1.5">
                <TeamMark team={r.team} />
                {#each r.players as p (p.slot)}
                    <HeroChip
                        heroId={p.heroId}
                        {heroes}
                        team={p.team}
                        size={22}
                        emphasised={p.slot === selected.slot}
                        dimmed={p.slot !== selected.slot}
                    />
                {/each}
            </div>
        {/each}
        <p class="flex flex-wrap items-center gap-x-4 text-xs text-muted-foreground">
            <span>{t("match_history.deep_dive.progress.player_hint")}</span>
            {#if playerReadout !== null && hoverT !== null}
                <span class="text-foreground">{clock(hoverT)} · {num(playerReadout)}</span>
            {/if}
        </p>
    </div>
</SectionCard>
