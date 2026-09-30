<script lang="ts">
    import Button from "$lib/ui/button.svelte";

    let { busy, onGo }: { busy: boolean; onGo: (target: string) => void } = $props();
</script>

<h1 class="font-heading text-3xl font-bold tracking-wide">You're set.</h1>
<p class="text-muted-foreground">Head to Home, or jump straight to a page.</p>
<div class="flex flex-wrap gap-2">
    <Button variant="outline" disabled={busy} onclick={() => onGo("/")}>Home</Button>
    <Button variant="outline" disabled={busy} onclick={() => onGo("/settings")}>Settings</Button>
    <Button variant="outline" disabled={busy} onclick={() => onGo("/whats-new")}>What's New</Button>
</div>
