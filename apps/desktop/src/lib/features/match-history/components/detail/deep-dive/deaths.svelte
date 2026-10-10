<script lang="ts">
    import { Bug, Clock, MapPin, Skull, Timer } from "@lucide/svelte";
    import { formatNumber, t } from "$lib/core/i18n.svelte";
    import type { Hero } from "$lib/features/heroes/heroes";
    import { formatClock } from "$lib/features/live/live";
    import { deathRows } from "../../../deep-dive/deaths";
    import type { MatchDetail, MatchPlayer } from "../../../detail";
    import HeroChip from "./hero-chip.svelte";
    import SectionCard from "./section-card.svelte";

    let { detail, selected, heroes }: { detail: MatchDetail; selected: MatchPlayer; heroes: Record<number, Hero> } =
        $props();

    const rows = $derived(deathRows(detail, selected));
    const clock = (s: number) => formatClock(Math.round(s)) ?? "";
</script>

<SectionCard title={t("match_history.deep_dive.deaths.heading")} icon={Skull} available={rows.length > 0}>
    <div class="overflow-x-auto">
        <table class="w-full text-sm">
            <thead class="text-left text-xs text-muted-foreground">
                <tr>
                    <th class="py-1 pr-3 font-normal">
                        <span class="inline-flex items-center gap-1">
                            <Clock size={14} aria-hidden="true" />
                            {t("match_history.deep_dive.deaths.time")}
                        </span>
                    </th>
                    <th class="py-1 pr-3 font-normal">
                        <span class="inline-flex items-center gap-1">
                            <Skull size={14} aria-hidden="true" />
                            {t("match_history.deep_dive.deaths.killer")}
                        </span>
                    </th>
                    <th class="py-1 pr-3 font-normal">
                        <span class="inline-flex items-center gap-1">
                            <Timer size={14} aria-hidden="true" />
                            {t("match_history.deep_dive.deaths.respawn")}
                        </span>
                    </th>
                    <th class="py-1 font-normal">
                        <span class="inline-flex items-center gap-1">
                            <MapPin size={14} aria-hidden="true" />
                            {t("match_history.deep_dive.deaths.position")}
                        </span>
                    </th>
                </tr>
            </thead>
            <tbody>
                {#each rows as r (r.timeS)}
                    <tr class="border-t border-border">
                        <td class="py-1.5 pr-3 tabular-nums">{clock(r.timeS)}</td>
                        <td class="py-1.5 pr-3">
                            {#if r.killerHeroId !== undefined}
                                <span class="inline-flex items-center gap-2">
                                    <Skull size={14} class="shrink-0 text-muted-foreground" aria-hidden="true" />
                                    <HeroChip heroId={r.killerHeroId} {heroes} team={r.killerTeam} size={24} />
                                </span>
                            {:else}
                                <span class="inline-flex items-center gap-2 text-muted-foreground">
                                    <Bug size={16} aria-hidden="true" />
                                    {t("match_history.deep_dive.deaths.other_killer", { slot: r.killerSlot })}
                                </span>
                            {/if}
                        </td>
                        <td class="py-1.5 pr-3 tabular-nums">
                            {r.respawnS === undefined
                                ? "–"
                                : t("match_history.deep_dive.deaths.seconds", {
                                      value: formatNumber(Math.round(r.respawnS)),
                                  })}
                        </td>
                        <td class="py-1.5 tabular-nums text-muted-foreground">
                            {r.position ? `${Math.round(r.position.x)}, ${Math.round(r.position.y)}` : "–"}
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
</SectionCard>
