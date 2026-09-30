<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import type { GameStatus } from "../../onboarding";

    let { returning, game }: { returning: boolean; game: GameStatus } = $props();
</script>

{#if returning}
    <h1 class="font-heading text-3xl font-bold tracking-wide">
        Deadlock+ has a revamped setup process, let's get you going.
    </h1>
    <p class="text-muted-foreground">You'll only see this once. Your settings stay as they are.</p>
{:else}
    <h1 class="font-heading text-3xl font-bold tracking-wide">Welcome to your new favourite Deadlock companion</h1>
    <p class="text-muted-foreground">
        Deadlock+ is ready to pick your servers, watch your connection and track every match.
    </p>
{/if}

<Card padding="sm" class="text-sm">
    {#if game.found}
        <p class="font-semibold">Deadlock found</p>
        <p class="break-all text-muted-foreground">{game.path}</p>
    {:else}
        <p class="font-semibold">Deadlock not found</p>
        <p class="text-muted-foreground">
            Install it through Steam. Replays, Mutes and Storage need the game folder. The rest works without it.
        </p>
    {/if}
</Card>
