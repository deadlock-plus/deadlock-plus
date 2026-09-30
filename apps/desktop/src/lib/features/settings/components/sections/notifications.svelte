<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import SettingRow from "$lib/ui/setting-row.svelte";
    import Input from "$lib/ui/input.svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { formatTime, LEAD_OPTIONS, parseTime, WEEKDAYS } from "$lib/features/settings/maintenance";

    let { show }: { show: (id: string) => boolean } = $props();

    let maintenanceTime = $state(formatTime(settings.maintenance.minuteOfDay));
    $effect(() => {
        maintenanceTime = formatTime(settings.maintenance.minuteOfDay);
    });

    function commitMaintenanceTime() {
        const minute = parseTime(maintenanceTime);
        if (minute === null) maintenanceTime = formatTime(settings.maintenance.minuteOfDay);
        else void settings.setMaintenance({ minuteOfDay: minute });
    }
</script>

{#if show("update-alerts")}
    <Card as="section">
        <SettingRow
            label="Patch and news alerts"
            for="update-alerts"
            description="Notifies you when a new Deadlock patch note or Steam announcement is posted. Checks the public Deadlock API every 15 minutes and sends nothing about you. Needs the app running, so pair it with the tray option."
        >
            <Switch
                id="update-alerts"
                checked={settings.updateAlerts}
                onCheckedChange={(v) => settings.setUpdateAlerts(v)}
            />
        </SettingRow>
    </Card>
{/if}

{#if show("maintenance")}
    <Card as="section">
        <SettingRow
            label="Steam maintenance reminder"
            for="maintenance"
            description="Notifies you before Steam's weekly maintenance, which can drop or restart servers. Valve publishes no schedule, so this is the usual slot (Tuesday evening, around 00:00 UTC Wednesday), not an official one. It usually lasts 15 to 30 minutes. Change it if yours differs. Needs the app running, so pair it with the tray option."
        >
            <Switch
                id="maintenance"
                checked={settings.maintenance.enabled}
                onCheckedChange={(v) => settings.setMaintenance({ enabled: v })}
            />
        </SettingRow>
        <div class="mt-3 flex flex-wrap items-center gap-3 text-sm">
            <label class="flex items-center gap-2 text-muted-foreground">
                Day
                <Select
                    value={settings.maintenance.weekday}
                    onchange={(e) => settings.setMaintenance({ weekday: Number(e.currentTarget.value) })}
                >
                    {#each WEEKDAYS as day, i (day)}
                        <option value={i}>{day}</option>
                    {/each}
                </Select>
            </label>
            <label class="flex items-center gap-2 text-muted-foreground">
                Time (UTC)
                <Input
                    class="w-24"
                    bind:value={maintenanceTime}
                    placeholder="21:00"
                    onblur={commitMaintenanceTime}
                    onkeydown={(e) => e.key === "Enter" && commitMaintenanceTime()}
                />
            </label>
            <label class="flex items-center gap-2 text-muted-foreground">
                Remind me
                <Select
                    value={settings.maintenance.leadMinutes}
                    onchange={(e) => settings.setMaintenance({ leadMinutes: Number(e.currentTarget.value) })}
                >
                    {#each LEAD_OPTIONS as m (m)}
                        <option value={m}>{m >= 60 ? `${m / 60} h` : `${m} min`} before</option>
                    {/each}
                </Select>
            </label>
        </div>
    </Card>
{/if}
