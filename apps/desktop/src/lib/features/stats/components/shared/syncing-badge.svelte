<script lang="ts">
    import { tn } from "$lib/core/i18n.svelte";
    let { count = 1 }: { count?: number } = $props();
</script>

<span
    class="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground"
    title={tn("stats.syncing_hint", count)}
>
    {tn("stats.syncing", count)}
</span>
