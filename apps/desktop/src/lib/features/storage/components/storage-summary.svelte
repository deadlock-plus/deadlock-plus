<script lang="ts">
    import { Eraser } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import { formatBytes } from "$lib/features/demos/demos";

    type Props = {
        total: number;
        canClear: number;
        disabled: boolean;
        onclear: () => void;
    };

    let { total, canClear, disabled, onclear }: Props = $props();
</script>

<Card as="section" radius="md" padding="none" class="flex flex-wrap items-center justify-between gap-4 px-5 py-4">
    <div class="flex gap-8">
        <div>
            <p class="text-xs text-muted-foreground">Total found</p>
            <p class="text-2xl tabular-nums">{formatBytes(total)}</p>
        </div>
        <div>
            <p class="text-xs text-muted-foreground">Can be cleared here</p>
            <p class="text-2xl tabular-nums">{formatBytes(canClear)}</p>
        </div>
    </div>
    <Button variant="outline" {disabled} onclick={onclear}>
        <Eraser />
        Clear regenerable
    </Button>
</Card>
