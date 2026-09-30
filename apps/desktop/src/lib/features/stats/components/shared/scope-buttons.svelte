<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import type { Scope } from "../../stats";

    let { scope = $bindable() }: { scope: Scope } = $props();

    const SCOPES: { id: Scope; label: string }[] = [
        { id: "ranked", label: "Ranked" },
        { id: "unranked", label: "Unranked" },
        { id: "all", label: "All modes" },
    ];
</script>

{#each SCOPES as s (s.id)}
    <Button size="sm" variant={scope === s.id ? "default" : "outline"} onclick={() => (scope = s.id)}>{s.label}</Button>
{/each}
