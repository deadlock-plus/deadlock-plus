<script lang="ts">
    import { onMount } from "svelte";
    import Switch from "$lib/components/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { autostartLine, getAutostart, setAutostart, type AutostartStatus } from "$lib/features/settings/autostart";

    let { show }: { show: (id: string) => boolean } = $props();

    let autostart = $state<AutostartStatus | null>(null);
    let autostartError = $state<string | null>(null);
    let autostartBusy = $state(false);

    onMount(() => {
        getAutostart()
            .then((a) => (autostart = a))
            .catch(() => {});
    });

    async function toggleAutostart(enabled: boolean) {
        autostartBusy = true;
        autostartError = null;
        try {
            autostart = await setAutostart(enabled);
        } catch (e) {
            autostartError = String(e);
        } finally {
            autostartBusy = false;
        }
    }
</script>

{#if show("autostart")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="autostart" class="font-heading text-sm font-semibold tracking-wide"
                    >Start with Windows</label
                >
                <p class="text-sm text-muted-foreground">Launches Deadlock+ when you sign in to Windows.</p>
                {#if autostartLine(autostart, autostartError)}
                    <p class="text-xs text-muted-foreground/80">{autostartLine(autostart, autostartError)}</p>
                {/if}
            </div>
            <Switch
                id="autostart"
                checked={autostart?.enabled ?? false}
                disabled={!autostart || autostartBusy}
                onCheckedChange={toggleAutostart}
            />
        </div>
    </section>
{/if}

{#if show("close-to-tray")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="close-to-tray" class="font-heading text-sm font-semibold tracking-wide"
                    >Keep running in the tray</label
                >
                <p class="text-sm text-muted-foreground">
                    Closing the window hides Deadlock+ instead of quitting, so monitoring keeps going. Quit from the
                    tray icon.
                </p>
            </div>
            <Switch
                id="close-to-tray"
                checked={settings.closeToTray}
                onCheckedChange={(v) => settings.setCloseToTray(v)}
            />
        </div>
    </section>
{/if}
