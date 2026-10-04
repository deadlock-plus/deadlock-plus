<script lang="ts">
    import { t } from "$lib/core/i18n.svelte";
    import Card from "$lib/ui/card.svelte";
    import SettingRow from "$lib/ui/setting-row.svelte";
    import Input from "$lib/ui/input.svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { formatTime, LEAD_OPTIONS, parseTime, weekdays } from "$lib/features/settings/maintenance";

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
            label={t("settings.items.update_alerts")}
            for="update-alerts"
            description={t("settings.notifications.alerts_description")}
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
            label={t("settings.items.maintenance")}
            for="maintenance"
            description={t("settings.notifications.maintenance_description")}
        >
            <Switch
                id="maintenance"
                checked={settings.maintenance.enabled}
                onCheckedChange={(v) => settings.setMaintenance({ enabled: v })}
            />
        </SettingRow>
        <div class="mt-3 flex flex-wrap items-center gap-3 text-sm">
            <label class="flex items-center gap-2 text-muted-foreground">
                {t("settings.notifications.day")}
                <Select
                    value={settings.maintenance.weekday}
                    onchange={(e) => settings.setMaintenance({ weekday: Number(e.currentTarget.value) })}
                >
                    {#each weekdays() as day, i (day)}
                        <option value={i}>{day}</option>
                    {/each}
                </Select>
            </label>
            <label class="flex items-center gap-2 text-muted-foreground">
                {t("settings.notifications.time")}
                <Input
                    class="w-24"
                    bind:value={maintenanceTime}
                    placeholder="21:00"
                    onblur={commitMaintenanceTime}
                    onkeydown={(e) => e.key === "Enter" && commitMaintenanceTime()}
                />
            </label>
            <label class="flex items-center gap-2 text-muted-foreground">
                {t("settings.notifications.remind")}
                <Select
                    value={settings.maintenance.leadMinutes}
                    onchange={(e) => settings.setMaintenance({ leadMinutes: Number(e.currentTarget.value) })}
                >
                    {#each LEAD_OPTIONS as m (m)}
                        <option value={m}
                            >{m >= 60
                                ? t("settings.notifications.lead_hours", { hours: m / 60 })
                                : t("settings.notifications.lead_minutes", { minutes: m })}</option
                        >
                    {/each}
                </Select>
            </label>
        </div>
    </Card>
{/if}
