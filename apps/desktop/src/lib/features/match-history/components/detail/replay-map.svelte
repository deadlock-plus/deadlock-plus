<script lang="ts">
    import { t } from "$lib/core/i18n.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import type { MinimapArt } from "$lib/features/gamedata/gamedata";
    import { formatClock } from "$lib/features/live/live";
    import type { DeathEvent, DeathMarker, MapDot } from "../../replay";
    import HeroChip from "./deep-dive/hero-chip.svelte";

    interface Props {
        art: MinimapArt | null;
        artLoaded: boolean;
        dots: MapDot[];
        heroOf: (slot: number) => number | null;
        heroes: Record<number, Hero>;
        nameOf: (slot: number) => string;
        heroNameOf: (slot: number) => string;
        ownSlot: number | null;
        deathMode: boolean;
        markers: DeathMarker[];
        selected: DeathEvent | null;
        onselect: (event: DeathEvent | null) => void;
    }

    let {
        art,
        artLoaded,
        dots,
        heroOf,
        heroes,
        nameOf,
        heroNameOf,
        ownSlot,
        deathMode,
        markers,
        selected,
        onselect,
    }: Props = $props();

    let focusSlot = $state<number | null>(null);

    const clock = (s: number) => formatClock(Math.round(s)) ?? "";
    const dotLabel = (slot: number) => `${nameOf(slot)} · ${heroNameOf(slot)}`;
    const selectedMarker = $derived(markers.find((m) => m.event === selected) ?? null);
    const focusDot = $derived(focusSlot === null ? null : (dots.find((d) => d.slot === focusSlot) ?? null));

    function anchor(left: number, top: number): string {
        const x = left < 22 ? "0%" : left > 78 ? "-100%" : "-50%";
        const y = top < 18 ? "14px" : "calc(-100% - 14px)";
        return `translate(${x}, ${y})`;
    }

    function onkeydown(e: KeyboardEvent) {
        if (e.key === "Escape" && selected) onselect(null);
    }
</script>

<svelte:window {onkeydown} />

<div
    class="relative mx-auto aspect-square w-full max-w-[560px] overflow-hidden rounded-lg border border-border bg-muted"
    role="group"
    aria-label={t("match_history.detail.replay.map_aria")}
>
    {#if art}
        <img src={art.src} alt="" draggable="false" class="absolute inset-0 size-full select-none object-fill" />
    {/if}

    {#if deathMode}
        <svg viewBox="0 0 100 100" preserveAspectRatio="none" class="absolute inset-0 size-full" aria-hidden="true">
            {#each markers as m, i (i)}
                {#if m.killer}
                    <line
                        x1={m.victim.left}
                        y1={m.victim.top}
                        x2={m.killer.left}
                        y2={m.killer.top}
                        stroke={m.event.victimTeam === "hidden-king" ? "var(--dd-hk)" : "var(--dd-am)"}
                        stroke-width={m.event === selected ? 2 : 1}
                        stroke-opacity={selected === null || m.event === selected ? 0.8 : 0.3}
                        stroke-dasharray="3 2"
                        vector-effect="non-scaling-stroke"
                    />
                {/if}
            {/each}
        </svg>
        {#each markers as m, i (i)}
            {@const killer = m.killer}
            {#if killer}
                <span
                    class="pointer-events-none absolute size-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-foreground/80"
                    style:left="{killer.left}%"
                    style:top="{killer.top}%"
                    class:opacity-30={selected !== null && m.event !== selected}
                    aria-hidden="true"
                ></span>
            {/if}
            <button
                type="button"
                class="death absolute size-3.5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                class:hk={m.event.victimTeam === "hidden-king"}
                class:am={m.event.victimTeam === "archmother"}
                class:picked={m.event === selected}
                style:left="{m.victim.left}%"
                style:top="{m.victim.top}%"
                aria-pressed={m.event === selected}
                aria-label={t("match_history.detail.replay.death_marker", {
                    victim: nameOf(m.event.victimSlot),
                    time: clock(m.event.timeS),
                })}
                onclick={() => onselect(m.event === selected ? null : m.event)}
            ></button>
        {/each}
        {#if selectedMarker}
            <div
                class="tip pointer-events-none absolute z-10 w-max max-w-48 rounded-md border border-border bg-popover px-2 py-1 text-xs text-popover-foreground shadow-md"
                style:left="{selectedMarker.victim.left}%"
                style:top="{selectedMarker.victim.top}%"
                style:transform={anchor(selectedMarker.victim.left, selectedMarker.victim.top)}
                role="status"
            >
                <div class="font-medium">{dotLabel(selectedMarker.event.victimSlot)}</div>
                <div class="text-muted-foreground">
                    {t("match_history.detail.replay.killed_by", { killer: nameOf(selectedMarker.event.killerSlot) })}
                </div>
                <div class="text-muted-foreground tabular-nums">{clock(selectedMarker.event.timeS)}</div>
            </div>
        {/if}
    {:else}
        {#each dots as d (d.slot)}
            {@const own = d.slot === ownSlot}
            <button
                type="button"
                class="dot absolute -translate-x-1/2 -translate-y-1/2 rounded-[8px] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                class:dead={!d.alive}
                class:own
                style:left="{d.left}%"
                style:top="{d.top}%"
                style:z-index={own ? 3 : d.alive ? 2 : 1}
                aria-label={d.alive
                    ? dotLabel(d.slot)
                    : t("match_history.detail.replay.dot_dead", { label: dotLabel(d.slot) })}
                onpointerenter={() => (focusSlot = d.slot)}
                onpointerleave={() => (focusSlot = null)}
                onfocus={() => (focusSlot = d.slot)}
                onblur={() => (focusSlot = null)}
            >
                <HeroChip
                    heroId={heroOf(d.slot)}
                    {heroes}
                    team={d.team}
                    size={own ? 30 : 24}
                    emphasised={own}
                    dimmed={!d.alive}
                />
            </button>
        {/each}
        {#if focusDot}
            <div
                class="pointer-events-none absolute z-10 w-max max-w-48 rounded-md border border-border bg-popover px-2 py-1 text-xs text-popover-foreground shadow-md"
                style:left="{focusDot.left}%"
                style:top="{focusDot.top}%"
                style:transform={anchor(focusDot.left, focusDot.top)}
                role="status"
            >
                <div class="font-medium">{nameOf(focusDot.slot)}</div>
                <div class="text-muted-foreground">
                    {heroNameOf(focusDot.slot)}{focusDot.alive ? "" : ` · ${t("match_history.detail.replay.dead")}`}
                </div>
            </div>
        {/if}
    {/if}

    {#if artLoaded && !art}
        <p class="pointer-events-none absolute inset-x-2 bottom-2 text-center text-xs text-muted-foreground">
            {t("match_history.detail.replay.no_art")}
        </p>
    {/if}
</div>

<style>
    .dot {
        transition: opacity 0.2s;
    }
    .dot.dead {
        opacity: 0.4;
        filter: grayscale(1);
    }
    .death {
        background: var(--color-muted-foreground);
        transition: transform 0.15s;
    }
    .death.hk {
        background: var(--dd-hk);
    }
    .death.am {
        background: var(--dd-am);
    }
    .death.picked {
        scale: 1.35;
        border-color: var(--color-foreground);
    }
    @media (prefers-reduced-motion: reduce) {
        .dot,
        .death {
            transition: none;
        }
    }
</style>
