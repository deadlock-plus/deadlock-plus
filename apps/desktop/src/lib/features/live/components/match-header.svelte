<script lang="ts">
    import { Pause } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
    import type { LiveMatch } from "$lib/generated/types/LiveMatch";
    import type { LivePhase } from "$lib/generated/types/LivePhase";
    import { formatClock } from "../live";

    let { match, phase }: { match: LiveMatch; phase: LivePhase } = $props();

    const pregame = $derived(phase === "pregame");
    const clock = $derived(pregame ? null : formatClock(match.clockSecs));
    const mode = $derived(
        [
            match.matchMode && match.matchMode !== "other" ? t(`live.mode.${match.matchMode}`) : null,
            match.gameMode && match.gameMode !== "normal" ? t(`live.game.${match.gameMode}`) : null,
        ]
            .filter(Boolean)
            .join(" · "),
    );
</script>

<div class="flex flex-wrap items-center gap-x-3.5 gap-y-1 px-0.5 text-[13px] text-muted-foreground tabular-nums">
    {#if clock}
        <span class="inline-flex items-center gap-1">
            {clock}
            {#if match.paused}<Pause class="size-3" aria-label={t("live.paused")} />{/if}
        </span>
    {/if}
    {#if mode}<span>{mode}</span>{/if}
</div>
