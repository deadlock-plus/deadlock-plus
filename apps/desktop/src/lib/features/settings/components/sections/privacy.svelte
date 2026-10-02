<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import SettingRow from "$lib/ui/setting-row.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";

    let { show }: { show: (id: string) => boolean } = $props();

    const ingestLine = $derived.by(() => {
        if (settings.ingestPromptPending) return "Waiting for your answer.";
        if (!settings.matchIngest) return "Off";
        const ingest = ingestStatus.status;
        if (!ingest) return "";
        if (!ingest.steamFound) return "Steam's HTTP cache wasn't found.";
        if (ingest.lastError) return `Last upload failed: ${ingest.lastError}`;
        return `Watching Steam's cache. ${ingest.submitted} submitted this session.`;
    });
</script>

{#if show("match-ingest")}
    <Card as="section">
        <SettingRow
            label="Share match data with Deadlock API"
            for="match-ingest"
            description="Reads Deadlock replay links from Steam's local HTTP cache and uploads the match IDs, replay salts and your Steam account ID to api.deadlock-api.com so the community database can fetch those matches. Nothing else is read or sent. Same behaviour as the open-source deadlock-api-ingest tool."
            hint={ingestLine}
        >
            <Switch
                id="match-ingest"
                checked={settings.matchIngest}
                onCheckedChange={(v) => settings.setMatchIngest(v)}
            />
        </SettingRow>
    </Card>
{/if}

{#if show("gc-recovery")}
    <Card as="section">
        <SettingRow
            label="Recover missing match salts through Steam"
            for="gc-recovery"
            description="Reads your saved Steam login on this PC, signs in to Steam as you and asks Deadlock's game servers for the replay salts of matches the community database is missing. Your login never leaves this PC. Only the match IDs, salts and your Steam account ID are sent to api.deadlock-api.com. Runs a few times an hour, never while Deadlock is open, and is limited to 40 matches per account per day. Same behaviour as the open-source deadlock-api-ingest tool."
            hint={settings.gcRecovery ? "On. Needs Steam to remember your login." : "Off"}
        >
            <Switch id="gc-recovery" checked={settings.gcRecovery} onCheckedChange={(v) => settings.setGcRecovery(v)} />
        </SettingRow>
    </Card>
{/if}
