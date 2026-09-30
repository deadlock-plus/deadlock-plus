<script lang="ts" module>
    export interface Extras {
        autostart: boolean;
        closeToTray: boolean;
        updateAlerts: boolean;
        maintenance: boolean;
    }
</script>

<script lang="ts">
    import Switch from "$lib/ui/switch.svelte";
    import Card from "$lib/ui/card.svelte";

    let { extras = $bindable(), autostartSupported }: { extras: Extras; autostartSupported: boolean } = $props();
</script>

{#snippet extra(id: string, label: string, note: string, checked: boolean, set: (v: boolean) => void)}
    <Card padding="sm" class="flex items-center justify-between gap-4">
        <div class="flex flex-col gap-0.5">
            <label for={id} class="font-heading text-sm font-semibold tracking-wide">{label}</label>
            <p class="text-xs text-muted-foreground">{note}</p>
        </div>
        <Switch {id} {checked} onCheckedChange={set} />
    </Card>
{/snippet}

<h1 class="font-heading text-2xl font-bold tracking-wide">Optional extras</h1>
<p class="text-muted-foreground">All off by default. Change any of them later in Settings.</p>
<div class="flex flex-col gap-2">
    {#if autostartSupported}
        {@render extra(
            "ob-autostart",
            "Start with Windows",
            "Launches Deadlock+ when you sign in.",
            extras.autostart,
            (v) => (extras.autostart = v),
        )}
    {/if}
    {@render extra(
        "ob-tray",
        "Keep running in the tray",
        "Closing the window hides it instead of quitting.",
        extras.closeToTray,
        (v) => (extras.closeToTray = v),
    )}
    {@render extra(
        "ob-alerts",
        "Patch and news alerts",
        "A notification when a new patch note or announcement is posted.",
        extras.updateAlerts,
        (v) => (extras.updateAlerts = v),
    )}
    {@render extra(
        "ob-maintenance",
        "Steam maintenance reminder",
        "A notification before the usual weekly maintenance.",
        extras.maintenance,
        (v) => (extras.maintenance = v),
    )}
</div>
<p class="text-xs text-muted-foreground">
    Alerts and reminders only fire while the app runs, so they pair well with the tray option.
</p>
