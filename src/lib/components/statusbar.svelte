<script lang="ts">
    import { onMount } from "svelte";
    import { CircleAlert, CloudUpload, CloudOff, Gamepad2 } from "@lucide/svelte";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { isGameRunning } from "$lib/features/voice-bans/api";

    const POLL_MS = 5000;

    let gameRunning = $state<boolean | null>(null);

    onMount(() => {
        const poll = () => {
            if (document.hidden) return;
            isGameRunning().then(
                (r) => (gameRunning = r),
                () => (gameRunning = null),
            );
        };
        poll();
        const timer = setInterval(poll, POLL_MS);
        return () => clearInterval(timer);
    });

    const s = $derived(ingestStatus.status);

    const ingest = $derived.by(() => {
        if (!settings.matchIngest)
            return { icon: CloudOff, tone: "text-muted-foreground/70", text: "Deadlock API ingest off" };
        if (!s) return { icon: CloudUpload, tone: "text-muted-foreground/70", text: "Deadlock API ingest starting..." };
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
    {#if gameRunning !== null}
        <span
            class="flex items-center gap-1.5 {gameRunning ? 'text-success' : 'text-muted-foreground/70'}"
            title={gameRunning ? "Deadlock is running" : "Deadlock is not running"}
        >
            <Gamepad2 class="size-3.5 shrink-0" />
            <span>{gameRunning ? "Deadlock running" : "Deadlock not running"}</span>
        </span>
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
    {/if}
    <span class="flex min-w-0 items-center gap-1.5 {ingest.tone}" title={ingest.text}>
        <Icon class="size-3.5 shrink-0" />
        <span class="truncate">{ingest.text}</span>
    </span>
</footer>
