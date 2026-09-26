<script lang="ts">
    import { CircleAlert, CloudUpload, CloudOff } from "@lucide/svelte";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";

    const s = $derived(ingestStatus.status);

    const ingest = $derived.by(() => {
        if (!settings.matchIngest)
            return { icon: CloudOff, tone: "text-muted-foreground/70", text: "Deadlock API ingest off" };
        if (!s) return { icon: CloudUpload, tone: "text-muted-foreground/70", text: "Deadlock API ingest starting…" };
        if (!s.steamFound)
            return { icon: CircleAlert, tone: "text-warning", text: "Deadlock API ingest: Steam cache not found" };
        if (s.lastError)
            return { icon: CircleAlert, tone: "text-destructive", text: `Deadlock API ingest: ${s.lastError}` };
        return {
            icon: CloudUpload,
            tone: "text-success",
            text: `Deadlock API ingest active · ${s.submitted} sent`,
        };
    });
    const Icon = $derived(ingest.icon);
</script>

<footer
    class="pointer-events-auto flex h-7 shrink-0 flex-row-reverse items-center gap-4 bg-chrome px-3 text-xs text-muted-foreground select-none"
>
    <span class="flex min-w-0 items-center gap-1.5 {ingest.tone}" title={ingest.text}>
        <Icon class="size-3.5 shrink-0" />
        <span class="truncate">{ingest.text}</span>
    </span>
</footer>
