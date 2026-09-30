<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import type { Scope } from "../../stats";
    import ScopeButtons from "../shared/scope-buttons.svelte";

    let { scope = $bindable(), days = $bindable() }: { scope: Scope; days: number | null } = $props();

    const WINDOWS: { days: number | null; label: string }[] = [
        { days: 7, label: "7 days" },
        { days: 30, label: "30 days" },
        { days: 90, label: "90 days" },
        { days: null, label: "All time" },
    ];
</script>

<div class="flex flex-wrap items-center gap-2">
    <ScopeButtons bind:scope />
    <span class="mx-2 h-5 w-px bg-border" aria-hidden="true"></span>
    {#each WINDOWS as w (w.label)}
        <Button size="sm" variant={days === w.days ? "default" : "outline"} onclick={() => (days = w.days)}
            >{w.label}</Button
        >
    {/each}
</div>
