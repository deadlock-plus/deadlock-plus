<script lang="ts">
    import { Search, Volume2 } from "@lucide/svelte";
    import Button from "$lib/ui/button.svelte";
    import Input from "$lib/ui/input.svelte";

    let {
        value = $bindable(),
        selectedCount,
        disabled,
        onunmute,
    }: { value: string; selectedCount: number; disabled: boolean; onunmute: () => void } = $props();
</script>

<div class="flex items-center gap-2">
    <div class="relative flex-1">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input bind:value placeholder="Filter by name or ID..." aria-label="Filter by name or ID" class="pl-8" />
    </div>
    <Button variant="outline" size="sm" disabled={selectedCount === 0 || disabled} onclick={onunmute}>
        <Volume2 />
        Unmute selected ({selectedCount})
    </Button>
</div>
