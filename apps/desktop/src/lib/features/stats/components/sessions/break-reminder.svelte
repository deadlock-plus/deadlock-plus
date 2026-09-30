<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";

    let { lossStreak, suggest }: { lossStreak: number; suggest: boolean } = $props();
</script>

<Card as="section">
    <div class="flex items-center justify-between gap-4">
        <div class="flex flex-col gap-1">
            <label for="break-hint" class="font-heading text-sm font-semibold tracking-wide">Break reminder</label>
            <p class="text-sm text-muted-foreground">
                Shows a note here when your current session has {lossStreak} losses in a row. Off by default.
            </p>
        </div>
        <Switch id="break-hint" checked={settings.breakHint} onCheckedChange={(v) => settings.setBreakHint(v)} />
    </div>
    {#if suggest}
        <p class="mt-3 rounded-md border border-border bg-muted px-3 py-2 text-sm">
            {lossStreak} losses in a row this session. A short break might be worth it. Up to you.
        </p>
    {/if}
</Card>
