<script lang="ts">
    import { Users } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
    import type { LiveQueue } from "$lib/generated/types/LiveQueue";
    import { formatClock } from "../live";

    let { queue }: { queue: LiveQueue } = $props();

    const time = $derived(formatClock(queue.queuedSecs));
</script>

<p class="flex flex-wrap items-center gap-x-3.5 text-sm text-muted-foreground tabular-nums">
    <span>{t("live.state.queuing")}</span>
    {#if time}<span>{time}</span>{/if}
    <span class="inline-flex items-center gap-1" title={t("live.queue.party")}>
        <Users class="size-3.5" />
        {queue.partySize}
    </span>
</p>
