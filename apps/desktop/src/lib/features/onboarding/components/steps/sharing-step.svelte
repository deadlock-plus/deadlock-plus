<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import type { IngestChoice } from "../../onboarding";

    let { choice, onAnswer }: { choice: IngestChoice | null; onAnswer: (choice: IngestChoice) => void } = $props();
</script>

<h1 class="font-heading text-2xl font-bold tracking-wide">Help the Deadlock community?</h1>
<p class="text-muted-foreground">
    The Deadlock API is a free, community-run database behind many stat and replay sites. Sharing your matches helps
    keep it complete for every player.
</p>
<p class="text-sm text-muted-foreground">
    If you agree, Deadlock+ sends only the match IDs and replay keys Steam already stores on your PC, plus your Steam
    account ID. Nothing else leaves your computer.
</p>
<p class="text-sm text-muted-foreground">Optional. You can change this any time in Settings.</p>
<div class="flex gap-2">
    <Button
        variant={choice === "share" ? "default" : "outline"}
        aria-pressed={choice === "share"}
        onclick={() => onAnswer("share")}
    >
        Share my matches
    </Button>
    <Button
        variant={choice === "decline" ? "default" : "outline"}
        aria-pressed={choice === "decline"}
        onclick={() => onAnswer("decline")}
    >
        No thanks
    </Button>
</div>
<p class="text-sm text-muted-foreground" role="status">
    {#if choice}
        {choice === "share" ? "Sharing is on." : "Sharing is off."} Saved.
    {:else}
        Skip this and sharing stays off until you decide in Settings.
    {/if}
</p>
