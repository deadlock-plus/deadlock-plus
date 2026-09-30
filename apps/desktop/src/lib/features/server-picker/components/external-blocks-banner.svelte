<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";

    let {
        ruleCount,
        regionCount,
        label,
        importing,
        onImport,
    }: {
        ruleCount: number;
        regionCount: number;
        label: string;
        importing: boolean;
        onImport: () => void;
    } = $props();
</script>

<Card radius="md" padding="none" class="flex items-center justify-between gap-3 px-3 py-2 text-sm">
    <span>
        Found {ruleCount}
        {label} rule{ruleCount === 1 ? "" : "s"} blocking
        {regionCount} region{regionCount === 1 ? "" : "s"}.
    </span>
    <Button size="sm" onclick={onImport} disabled={importing}>Import</Button>
</Card>
