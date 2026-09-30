<script lang="ts">
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
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="match-ingest" class="font-heading text-sm font-semibold tracking-wide">
                    Share match data with Deadlock API
                </label>
                <p class="text-sm text-muted-foreground">
                    Reads Deadlock replay links from Steam's local HTTP cache and uploads the match IDs, replay salts
                    and your Steam account ID to api.deadlock-api.com so the community database can fetch those matches.
                    Nothing else is read or sent. Same behaviour as the open-source deadlock-api-ingest tool.
                </p>
                {#if ingestLine}<p class="text-xs text-muted-foreground/80">{ingestLine}</p>{/if}
            </div>
            <Switch
                id="match-ingest"
                checked={settings.matchIngest}
                onCheckedChange={(v) => settings.setMatchIngest(v)}
            />
        </div>
    </section>
{/if}
