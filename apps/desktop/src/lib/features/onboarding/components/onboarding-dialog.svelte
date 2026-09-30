<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "$lib/ui/alert-dialog";
    import Button from "$lib/ui/button.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { getAutostart, setAutostart } from "$lib/features/settings/autostart";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { platform, platformName } from "$lib/core/platform";
    import { onboarding } from "../onboarding.svelte";

    const LAST_STEP = 2;

    let step = $state(0);
    let autostart = $state(false);
    let autostartSupported = $state(true);
    let closeToTray = $state(false);
    let updateAlerts = $state(false);
    let maintenance = $state(false);
    let busy = $state(false);

    onMount(() => {
        getAutostart()
            .then((a) => (autostartSupported = a.supported))
            .catch(() => {});
    });

    async function finish() {
        busy = true;
        try {
            if (autostart && autostartSupported) {
                try {
                    await setAutostart(true);
                } catch (e) {
                    toast.error(`Couldn't turn on Start with Windows: ${e}`);
                }
            }
            if (closeToTray) await settings.setCloseToTray(true);
            if (updateAlerts) await settings.setUpdateAlerts(true);
            if (maintenance) await settings.setMaintenance({ enabled: true });
            await onboarding.finish();
        } finally {
            busy = false;
        }
    }
</script>

{#snippet extra(id: string, label: string, note: string, checked: boolean, set: (v: boolean) => void)}
    <div class="flex items-center justify-between gap-4 rounded-lg border bg-card p-3">
        <div class="flex flex-col gap-0.5">
            <label for={id} class="font-heading text-sm font-semibold tracking-wide">{label}</label>
            <p class="text-xs text-muted-foreground">{note}</p>
        </div>
        <Switch {id} {checked} onCheckedChange={set} />
    </div>
{/snippet}

<AlertDialog.Root open={onboarding.open} onOpenChange={(o) => !o && onboarding.finish()}>
    <AlertDialog.Content class="max-w-lg">
        <div class="flex flex-col gap-3">
            {#if step === 0}
                <AlertDialog.Title>Welcome to Deadlock+</AlertDialog.Title>
                <AlertDialog.Description>
                    Deadlock+ picks game servers, monitors your connection and tracks your matches. This quick tour
                    covers what it needs.
                </AlertDialog.Description>
                <div class="rounded-lg border bg-card p-3 text-sm">
                    {#if onboarding.game.found}
                        <p class="font-semibold">Deadlock found</p>
                        <p class="break-all text-muted-foreground">{onboarding.game.path}</p>
                    {:else}
                        <p class="font-semibold">Deadlock not found</p>
                        <p class="text-muted-foreground">
                            Install it through Steam. Replays, Mutes and Storage need the game folder. The rest works
                            without it.
                        </p>
                    {/if}
                </div>
            {:else if step === 1 && platform !== "windows"}
                <AlertDialog.Title>Running on {platformName(platform)}</AlertDialog.Title>
                <AlertDialog.Description>
                    {platformName(platform)} support is best-effort and mostly untested.
                </AlertDialog.Description>
                <ul class="flex list-disc flex-col gap-1.5 pl-5 text-sm text-muted-foreground">
                    <li>
                        <span class="text-foreground">Not available yet:</span> Frametimes.
                    </li>
                    <li>
                        <span class="text-foreground">Untested:</span> the Server Picker and the Connection page. Both ask
                        for your password.
                    </li>
                    <li>
                        <span class="text-foreground">Something broken?</span> Please report it on GitHub. A fix is a big
                        plus.
                    </li>
                </ul>
            {:else if step === 1}
                <AlertDialog.Title>Why Windows asks for permission</AlertDialog.Title>
                <AlertDialog.Description>
                    Deadlock+ runs as administrator, so Windows shows a UAC prompt on every launch.
                </AlertDialog.Description>
                <ul class="flex list-disc flex-col gap-1.5 pl-5 text-sm text-muted-foreground">
                    <li>
                        <span class="text-foreground">Firewall rules:</span> the Server Picker blocks the regions you
                        pick by adding Windows Firewall rules named <code>deadlock_plus_*</code>. Only administrators
                        can do that.
                    </li>
                    <li>
                        <span class="text-foreground">Connection monitor:</span> live ping and loss for your match come from
                        Windows network tracing, which is also admin-only.
                    </li>
                    <li>Nothing else needs elevated rights.</li>
                </ul>
                {#if !onboarding.elevated}
                    <p class="text-sm text-muted-foreground">
                        This copy runs without administrator rights, so the Server Picker and Connection page won't work
                        until you relaunch it elevated.
                    </p>
                {/if}
            {:else}
                <AlertDialog.Title>Optional extras</AlertDialog.Title>
                <AlertDialog.Description
                    >All off by default. Change any of them later in Settings.</AlertDialog.Description
                >
                <div class="flex flex-col gap-2">
                    {#if autostartSupported}
                        {@render extra(
                            "ob-autostart",
                            "Start with Windows",
                            "Launches Deadlock+ when you sign in.",
                            autostart,
                            (v) => (autostart = v),
                        )}
                    {/if}
                    {@render extra(
                        "ob-tray",
                        "Keep running in the tray",
                        "Closing the window hides it instead of quitting.",
                        closeToTray,
                        (v) => (closeToTray = v),
                    )}
                    {@render extra(
                        "ob-alerts",
                        "Patch and news alerts",
                        "A notification when a new patch note or announcement is posted.",
                        updateAlerts,
                        (v) => (updateAlerts = v),
                    )}
                    {@render extra(
                        "ob-maintenance",
                        "Steam maintenance reminder",
                        "A notification before the usual weekly maintenance.",
                        maintenance,
                        (v) => (maintenance = v),
                    )}
                </div>
                <p class="text-xs text-muted-foreground">
                    Alerts and reminders only fire while the app runs, so they pair well with the tray option.
                </p>
            {/if}
        </div>
        <AlertDialog.Footer>
            {#if step === 0}
                <Button variant="ghost" onclick={() => onboarding.finish()}>Skip</Button>
            {:else}
                <Button variant="ghost" disabled={busy} onclick={() => (step -= 1)}>Back</Button>
            {/if}
            {#if step < LAST_STEP}
                <Button onclick={() => (step += 1)}>Next</Button>
            {:else}
                <Button disabled={busy} onclick={finish}>Finish</Button>
            {/if}
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
