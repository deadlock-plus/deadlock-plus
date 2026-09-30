<script lang="ts">
    import { ChevronLeft, ChevronRight } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";

    let { page = $bindable(0), count }: { page?: number; count: number } = $props();
</script>

<div class="flex items-center justify-center gap-3">
    <Button variant="outline" size="sm" disabled={page === 0} onclick={() => page--} aria-label="Previous page">
        <ChevronLeft />
    </Button>
    <span class="text-sm text-muted-foreground">Page {page + 1} of {count}</span>
    <Button variant="outline" size="sm" disabled={page >= count - 1} onclick={() => page++} aria-label="Next page">
        <ChevronRight />
    </Button>
</div>
