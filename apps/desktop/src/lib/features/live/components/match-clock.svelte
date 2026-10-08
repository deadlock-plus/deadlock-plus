<script lang="ts">
    import { Pause } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
    import { formatClock } from "../live";

    let { clockSecs, paused }: { clockSecs: number | null; paused: boolean } = $props();

    const clock = $derived(formatClock(clockSecs));
</script>

{#if clock}
    <span class="inline-flex items-center gap-1">
        {clock}
        {#if paused}<Pause class="size-3" aria-label={t("live.paused")} />{/if}
    </span>
{/if}
