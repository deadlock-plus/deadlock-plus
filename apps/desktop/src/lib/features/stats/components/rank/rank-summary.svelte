<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import type { RankInfo, RankTier, Standing } from "../../rank";
    import SyncingBadge from "../shared/syncing-badge.svelte";
    import ShieldPanel from "./shield-panel.svelte";

    let {
        info,
        now,
        tier,
        name,
        nextName,
        atTop,
        toNext,
        modelled = 0,
    }: {
        info: RankInfo;
        now: Standing;
        tier: RankTier | undefined;
        name: string;
        nextName: string;
        atTop: boolean;
        toNext: number | null;
        modelled?: number;
    } = $props();
</script>

<Card as="section" padding="lg" class="grid gap-4 md:grid-cols-[1fr_auto]">
    <div class="flex items-center gap-5">
        {#if tier?.image}
            <img src={tier.image} alt="" class="size-28 shrink-0 object-contain" />
        {/if}
        <div class="min-w-0 flex-1">
            <p class="flex items-center gap-2 text-sm text-muted-foreground">
                Current rank
                {#if modelled > 0}<SyncingBadge count={modelled} />{/if}
            </p>
            {#if modelled > 0}
                <p class="text-xs text-muted-foreground">
                    Estimated from the rank rules until the API confirms {modelled === 1
                        ? "your last match"
                        : "your last matches"}.
                </p>
            {/if}
            <p class="font-heading text-4xl font-semibold" style:color={tier?.color}>{name}</p>
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
                        Your progress just crossed a boundary and the API still shows the old badge, so there is no bar
                        until the next match.
                    </p>
                {/if}
            {/if}
            {#if (info.placementLeft ?? 0) > 0}
                <p class="mt-1 text-sm text-muted-foreground">{info.placementLeft} placement games left.</p>
            {/if}
        </div>
    </div>

    <ShieldPanel shieldsLeft={info.shieldsLeft} />
</Card>
