<script lang="ts">
    import { ChevronLeft, ChevronRight } from "@lucide/svelte";
    import IconButton from "$lib/ui/icon-button.svelte";

    let { page = $bindable(), pageCount }: { page: number; pageCount: number } = $props();
</script>

<div class="flex items-center justify-center gap-3">
    <IconButton variant="outline" label="Previous page" disabled={page === 0} onclick={() => page--}>
        <ChevronLeft />
    </IconButton>
    <span class="text-sm text-muted-foreground">Page {page + 1} of {pageCount}</span>
    <IconButton variant="outline" label="Next page" disabled={page >= pageCount - 1} onclick={() => page++}>
        <ChevronRight />
    </IconButton>
</div>
