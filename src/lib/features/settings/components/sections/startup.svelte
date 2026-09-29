<script lang="ts">
    import { onMount } from "svelte";
    import Select from "$lib/components/ui/select.svelte";
    import Switch from "$lib/components/ui/switch.svelte";
    import { POLICY_OPTIONS, type Policy } from "$lib/features/jobs/jobs";
    import { jobs } from "$lib/features/jobs/jobs.svelte";
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

{#if show("background-jobs")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex flex-col gap-1">
            <h3 class="font-heading text-sm font-semibold tracking-wide">Background work</h3>
            <p class="text-sm text-muted-foreground">
                Deadlock+ scans your addons and indexes patch notes in the background. These settings decide whether
                that work runs, and what it does while Deadlock is open.
            </p>
        </div>

        <div class="mt-4 flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="all-jobs" class="text-sm font-medium">Allow background work</label>
                <p class="text-xs text-muted-foreground">
                    Turn this off and no task starts by itself. You can still run a scan from the Performance page.
                </p>
            </div>
            <Switch id="all-jobs" checked={jobs.allEnabled} onCheckedChange={(v) => void jobs.setAllEnabled(v)} />
        </div>

        <div class="mt-4 flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="pause-in-game" class="text-sm font-medium">Pause background work while Deadlock runs</label>
                <p class="text-xs text-muted-foreground">
                    The app-wide switch for pausing. When it is on, every task set to Pause below stops while Deadlock
                    is open and carries on when you close it. When it is off, no task pauses, whatever it is set to.
                    Tasks set to Slow down still slow down.
                </p>
            </div>
            <Switch
                id="pause-in-game"
                checked={jobs.pauseInGame}
                disabled={!jobs.allEnabled}
                onCheckedChange={(v) => void jobs.setPauseInGame(v)}
            />
        </div>

        {#if jobs.catalog.length > 0}
            <div class="my-4 border-t" role="separator"></div>

            <div class="flex flex-col gap-1">
                <h4 class="text-sm font-medium">Tasks</h4>
                <p class="text-xs text-muted-foreground">
                    Each task can run by itself or not, and can pause, slow down or keep running while Deadlock is open.
                    Pause only works while the pause switch above is on.
                </p>
            </div>

            <ul class="mt-4 flex flex-col gap-5">
                {#each jobs.catalog as job (job.id)}
                    <li class="flex flex-col gap-2">
                        <div class="flex flex-col gap-0.5">
                            <span class="text-sm font-medium">{job.title}</span>
                            <p class="text-xs text-muted-foreground">{job.description}</p>
                        </div>
                        <div class="flex flex-wrap items-center gap-x-6 gap-y-2">
                            <div class="flex items-center gap-2">
                                <Switch
                                    id="job-{job.id}"
                                    checked={job.enabled}
                                    disabled={!jobs.allEnabled}
                                    onCheckedChange={(v) => void jobs.setEnabled(job.id, v)}
                                />
                                <label for="job-{job.id}" class="text-xs text-muted-foreground">Run by itself</label>
                            </div>
                            {#if job.policyConfigurable}
                                <div class="flex items-center gap-2">
                                    <label for="job-{job.id}-policy" class="text-xs text-muted-foreground"
                                        >While Deadlock runs</label
                                    >
                                    <Select
                                        id="job-{job.id}-policy"
                                        value={job.policy}
                                        disabled={!jobs.allEnabled || !job.enabled}
                                        onchange={(e) => void jobs.setPolicy(job.id, e.currentTarget.value as Policy)}
                                    >
                                        {#each POLICY_OPTIONS as option (option.value)}
                                            <option value={option.value}>{option.label}</option>
                                        {/each}
                                    </Select>
                                </div>
                            {/if}
                        </div>
                        {#if job.policyConfigurable && !jobs.pauseInGame && job.policy === "pauseInGame"}
                            <p class="text-xs text-muted-foreground/80">
                                Set to Pause, but the pause switch above is off, so this keeps running.
                            </p>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </section>
{/if}
