<script lang="ts" module>
    export interface Extras {
        autostart: boolean;
        closeToTray: boolean;
        updateAlerts: boolean;
        maintenance: boolean;
        gcRecovery: boolean;
        postgameCapture: boolean;
    }
</script>

<script lang="ts">
    import Switch from "$lib/ui/switch.svelte";
    import Card from "$lib/ui/card.svelte";
    import { autostartTitle } from "$lib/features/settings/autostart";
    import { platform } from "$lib/core/platform";
    import { matchDataExtras } from "../../onboarding";

    let { extras = $bindable(), autostartSupported }: { extras: Extras; autostartSupported: boolean } = $props();

    const matchData = matchDataExtras(platform);
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
            autostartTitle(),
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
    {#if matchData.includes("gcRecovery")}
        {@render extra(
            "ob-gc-recovery",
            "Recover missing match salts through Steam",
            "Uses your saved Steam login to fetch replay salts the community database is missing.",
            extras.gcRecovery,
            (v) => (extras.gcRecovery = v),
        )}
    {/if}
    {#if matchData.includes("postgameCapture")}
        {@render extra(
            "ob-postgame-capture",
            "Instant match results",
            "Finished matches show in Stats and Sessions right away, without waiting for the Deadlock API.",
            extras.postgameCapture,
            (v) => (extras.postgameCapture = v),
        )}
    {/if}
</div>
<p class="text-xs text-muted-foreground">
    Alerts and reminders only fire while the app runs, so they pair well with the tray option.
</p>
