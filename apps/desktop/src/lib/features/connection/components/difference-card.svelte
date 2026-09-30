<script lang="ts">
    import Card from "$lib/ui/card.svelte";

    type Props = {
        saved: number | null;
    };

    let { saved }: Props = $props();
</script>

<Card class="flex flex-col gap-3">
    <span class="text-sm font-medium">ExitLag difference</span>
    {#if saved == null}
        <p class="py-6 text-sm text-muted-foreground">Needs both pings.</p>
    {:else}
        <div class="flex items-baseline gap-1">
            <span class="text-4xl font-semibold tabular-nums {saved >= 0 ? 'text-success' : 'text-destructive'}">
                {saved >= 0 ? "−" : "+"}{Math.abs(Math.round(saved))}
            </span>
            <span class="text-sm text-muted-foreground">ms</span>
        </div>
        <p class="text-xs text-muted-foreground">
            {saved >= 0 ? "ExitLag is faster than your direct route." : "Your direct route is faster than ExitLag."}
        </p>
    {/if}
</Card>
