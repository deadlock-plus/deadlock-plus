<script lang="ts">
    import { untrack } from "svelte";
    import { Map as MapIcon, Pause, Play, Skull } from "@lucide/svelte";
    import { formatNumber, i18n, t } from "$lib/core/i18n.svelte";
    import { loadMinimapArt, type MinimapArt } from "$lib/features/gamedata/gamedata";
    import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
    import { formatClock } from "$lib/features/live/live";
    import Button from "$lib/ui/button.svelte";
    import Section from "$lib/ui/section.svelte";
    import Select from "$lib/ui/select.svelte";
    import { resolveItemListVisuals, type IdVisual } from "../../deep-dive/catalog";
    import { objectiveNameKey, objectiveRows } from "../../deep-dive/objective-rows";
    import { allPlayers, findPlayer, playerBySlot, type MatchDetail, type MatchTeam } from "../../detail";
    import { decodePaths, WORLD_RADIUS } from "../../paths";
    import {
        DEFAULT_KINDS,
        SPEEDS,
        advance,
        deathMarkers,
        mapDots,
        replayDuration,
        sampleIndex,
        type DeathEvent,
        type ReplaySpeed,
    } from "../../replay";
    import { buildTimeline, type TimelineEvent, type TimelineKind } from "../../timeline";
    import ReplayMap from "./replay-map.svelte";
    import ReplayTimeline from "./replay-timeline.svelte";

    let { detail, ownAccountIds = [] }: { detail: MatchDetail; ownAccountIds?: number[] } = $props();

    const MAX_FRAME_MS = 250;

    const decoded = $derived(decodePaths(detail));
    const events = $derived(buildTimeline(detail));
    const duration = $derived(replayDuration(detail, decoded, events));
    const players = $derived(allPlayers(detail));
    const ownSlot = $derived(findPlayer(detail, ownAccountIds)?.slot ?? null);

    let time = $state(0);
    let playing = $state(false);
    let speed = $state<ReplaySpeed>(1);
    let kinds = $state<TimelineKind[]>([...DEFAULT_KINDS]);
    let deathMode = $state(false);
    let filterSlot = $state<number | null>(null);
    let filterRole = $state<"victim" | "killer">("victim");
    let rangeFrom = $state(0);
    let rangeTo = $state(0);
    let rangeTouched = $state(false);
    let selected = $state.raw<TimelineEvent | null>(null);

    let art = $state.raw<MinimapArt | null>(null);
    let artLoaded = $state(false);
    let heroes = $state.raw<Record<number, Hero>>({});
    let itemVisuals = $state.raw(new Map<number, IdVisual>());

    $effect(() => {
        detail.matchId;
        time = 0;
        playing = false;
        selected = null;
        filterSlot = null;
        rangeTouched = false;
    });

    $effect(() => {
        let live = true;
        void loadMinimapArt().then((found) => {
            if (!live) return;
            art = found;
            artLoaded = true;
        });
        return () => (live = false);
    });

    $effect(() => {
        const ids = players.map((p) => p.heroId);
        let live = true;
        void loadHeroes(ids).then((h) => {
            if (live) heroes = h;
        });
        return () => (live = false);
    });

    const wantsItems = $derived(kinds.includes("item-buy") || kinds.includes("item-sell"));
    $effect(() => {
        if (!wantsItems) return;
        const ids = [
            ...new Set(events.flatMap((e) => (e.kind === "item-buy" || e.kind === "item-sell" ? [e.itemId] : []))),
        ];
        const locale = i18n.locale;
        let live = true;
        void resolveItemListVisuals(ids, locale).then((found) => {
            if (live) itemVisuals = found;
        });
        return () => (live = false);
    });

    $effect(() => {
        if (!playing) return;
        let last = performance.now();
        let raf = 0;
        const step = (now: number) => {
            const next = advance(
                untrack(() => time),
                Math.min(now - last, MAX_FRAME_MS),
                untrack(() => speed),
                untrack(() => duration),
            );
            last = now;
            time = next.t;
            if (next.ended) {
                playing = false;
                return;
            }
            raf = requestAnimationFrame(step);
        };
        raf = requestAnimationFrame(step);
        return () => cancelAnimationFrame(raf);
    });

    const radius = $derived(art?.radius ?? WORLD_RADIUS);
    const index = $derived(decoded ? sampleIndex(time, decoded.intervalS) : 0);
    const dots = $derived(decoded ? mapDots(decoded, index * decoded.intervalS, radius) : []);
    const effectiveTo = $derived(rangeTouched ? Math.min(rangeTo, duration) : duration);
    const effectiveFrom = $derived(rangeTouched ? Math.min(rangeFrom, effectiveTo) : 0);
    const markers = $derived(
        deathMode
            ? deathMarkers(
                  events,
                  {
                      slot: filterSlot ?? undefined,
                      role: filterRole,
                      fromS: effectiveFrom,
                      toS: effectiveTo,
                  },
                  radius,
              )
            : [],
    );

    const objectiveLabels = $derived.by(() => {
        const out = new Map<string, string>();
        for (const row of objectiveRows(detail.objectives)) {
            const name = t(objectiveNameKey(row.kind));
            out.set(
                `${row.objectiveId}:${row.team}:${row.destroyedS}`,
                row.index === undefined
                    ? name
                    : t("match_history.deep_dive.objectives.indexed_name", { name, index: row.index }),
            );
        }
        return out;
    });

    const clock = (s: number) => formatClock(Math.round(s)) ?? "0:00";
    const teamName = (team: MatchTeam | undefined) =>
        team === undefined
            ? t("match_history.detail.replay.unknown")
            : team === "hidden-king"
              ? t("match_history.team.hidden_king")
              : t("match_history.team.archmother");
    const nameOf = (slot: number) => {
        const p = playerBySlot(detail, slot);
        return p
            ? (p.name ?? t("match_history.deep_dive.player_fallback", { slot }))
            : t("match_history.detail.replay.unknown");
    };
    const heroOf = (slot: number) => playerBySlot(detail, slot)?.heroId ?? null;
    const heroNameOf = (slot: number) => {
        const id = heroOf(slot);
        return (id === null ? undefined : heroes[id]?.name) ?? t("match_history.detail.replay.unknown_hero");
    };
    const itemName = (id: number) => itemVisuals.get(id)?.name ?? t("match_history.deep_dive.names.item");

    function labelOf(e: TimelineEvent): string {
        const time = clock(e.timeS);
        switch (e.kind) {
            case "death":
                return t("match_history.detail.replay.marker_death", {
                    victim: nameOf(e.victimSlot),
                    killer: nameOf(e.killerSlot),
                    time,
                });
            case "objective":
                return t("match_history.detail.replay.marker_objective", {
                    objective:
                        objectiveLabels.get(`${e.objectiveId}:${e.team}:${e.timeS}`) ??
                        t("match_history.deep_dive.objectives.kind_unknown"),
                    team: teamName(e.team),
                    time,
                });
            case "mid-boss":
                return t("match_history.detail.replay.marker_mid_boss", {
                    team: teamName(e.claimedBy ?? e.killedBy),
                    time,
                });
            case "item-buy":
                return t("match_history.detail.replay.marker_item_buy", {
                    player: nameOf(e.slot),
                    item: itemName(e.itemId),
                    time,
                });
            case "item-sell":
                return t("match_history.detail.replay.marker_item_sell", {
                    player: nameOf(e.slot),
                    item: itemName(e.itemId),
                    time,
                });
            case "swing":
                return t("match_history.detail.replay.marker_swing", {
                    team: teamName(e.team),
                    souls: formatNumber(Math.abs(e.leadAfter - e.leadBefore)),
                    time,
                });
        }
    }

    function togglePlay() {
        if (!playing && time >= duration) time = 0;
        playing = !playing;
    }

    function onseek(timeS: number, event: TimelineEvent) {
        time = Math.min(Math.max(timeS, 0), duration);
        selected = event.kind === "death" ? event : null;
    }

    function onselect(event: DeathEvent | null) {
        selected = event;
    }

    function setFrom(value: number) {
        rangeTouched = true;
        rangeTo = effectiveTo;
        rangeFrom = Math.min(value, effectiveTo);
    }

    function setTo(value: number) {
        rangeTouched = true;
        rangeFrom = effectiveFrom;
        rangeTo = Math.max(value, effectiveFrom);
    }
</script>

<Section class="replay flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
        <h2 class="flex items-center gap-2 text-sm font-medium">
            <MapIcon size={16} class="shrink-0 text-muted-foreground" aria-hidden="true" />
            {t("match_history.detail.replay.heading")}
        </h2>
        {#if decoded}
            <Button
                size="sm"
                variant={deathMode ? "default" : "outline"}
                aria-pressed={deathMode}
                onclick={() => {
                    deathMode = !deathMode;
                    selected = null;
                }}
            >
                <Skull size={14} aria-hidden="true" />
                {t("match_history.detail.replay.death_map")}
            </Button>
        {/if}
    </div>

    {#if decoded}
        {#if deathMode}
            <div class="flex flex-wrap items-end gap-x-4 gap-y-2 text-sm">
                <label class="flex flex-col gap-1 text-xs text-muted-foreground">
                    {t("match_history.detail.replay.filter_player")}
                    <Select
                        value={filterSlot ?? ""}
                        onchange={(e) => {
                            filterSlot = e.currentTarget.value === "" ? null : Number(e.currentTarget.value);
                            selected = null;
                        }}
                    >
                        <option value="">{t("match_history.detail.replay.all_players")}</option>
                        {#each players as p (p.slot)}
                            <option value={p.slot}>{nameOf(p.slot)} · {heroNameOf(p.slot)}</option>
                        {/each}
                    </Select>
                </label>
                {#if filterSlot !== null}
                    <label class="flex flex-col gap-1 text-xs text-muted-foreground">
                        {t("match_history.detail.replay.filter_role")}
                        <Select
                            value={filterRole}
                            onchange={(e) => {
                                filterRole = e.currentTarget.value === "killer" ? "killer" : "victim";
                                selected = null;
                            }}
                        >
                            <option value="victim">{t("match_history.detail.replay.role_victim")}</option>
                            <option value="killer">{t("match_history.detail.replay.role_killer")}</option>
                        </Select>
                    </label>
                {/if}
                <label class="flex min-w-36 flex-1 flex-col gap-1 text-xs text-muted-foreground">
                    {t("match_history.detail.replay.range_from", { time: clock(effectiveFrom) })}
                    <input
                        type="range"
                        min="0"
                        max={duration}
                        step="1"
                        value={effectiveFrom}
                        class="accent-primary"
                        oninput={(e) => setFrom(Number(e.currentTarget.value))}
                    />
                </label>
                <label class="flex min-w-36 flex-1 flex-col gap-1 text-xs text-muted-foreground">
                    {t("match_history.detail.replay.range_to", { time: clock(effectiveTo) })}
                    <input
                        type="range"
                        min="0"
                        max={duration}
                        step="1"
                        value={effectiveTo}
                        class="accent-primary"
                        oninput={(e) => setTo(Number(e.currentTarget.value))}
                    />
                </label>
            </div>
            <p class="text-xs text-muted-foreground" role="status">
                {t("match_history.detail.replay.deaths_shown", { count: markers.length })}
            </p>
        {/if}

        <ReplayMap
            {art}
            {artLoaded}
            {dots}
            {heroOf}
            {heroes}
            {nameOf}
            {heroNameOf}
            {ownSlot}
            {deathMode}
            {markers}
            selected={selected?.kind === "death" ? selected : null}
            {onselect}
        />
    {:else}
        <p class="rounded-md border border-dashed border-border px-3 py-4 text-sm text-muted-foreground" role="status">
            {t("match_history.detail.replay.no_paths")}
        </p>
    {/if}

    <div class="flex flex-wrap items-center gap-2">
        <Button size="sm" variant="outline" aria-pressed={playing} onclick={togglePlay} disabled={duration <= 0}>
            {#if playing}
                <Pause aria-hidden="true" />
                {t("match_history.detail.replay.pause")}
            {:else}
                <Play aria-hidden="true" />
                {t("match_history.detail.replay.play")}
            {/if}
        </Button>
        <div class="flex items-center gap-1" role="group" aria-label={t("match_history.detail.replay.speed_aria")}>
            {#each SPEEDS as s (s)}
                <Button
                    size="sm"
                    variant={speed === s ? "default" : "outline"}
                    aria-pressed={speed === s}
                    onclick={() => (speed = s)}
                >
                    {t("match_history.detail.replay.speed", { speed: s })}
                </Button>
            {/each}
        </div>
        <span class="ml-auto text-sm tabular-nums text-muted-foreground">
            {t("match_history.detail.replay.time", { current: clock(time), total: clock(duration) })}
        </span>
    </div>

    {#if events.length === 0}
        <p class="text-sm text-muted-foreground" role="status">{t("match_history.detail.replay.no_events")}</p>
    {:else}
        <ReplayTimeline
            {events}
            {kinds}
            {duration}
            {time}
            {labelOf}
            {selected}
            onkinds={(next) => (kinds = next)}
            {onseek}
            onscrub={(s) => (time = s)}
        />
    {/if}
</Section>

<style>
    :global(.replay) {
        --dd-hk: oklch(0.8 0.125 82);
        --dd-am: oklch(0.72 0.11 245);
    }
    :global(:root[data-theme="daylight"] .replay) {
        --dd-hk: oklch(0.6 0.12 70);
        --dd-am: oklch(0.5 0.12 245);
    }
</style>
