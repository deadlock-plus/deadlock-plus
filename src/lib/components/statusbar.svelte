<script lang="ts">
    import { onMount } from "svelte";
    import { CircleAlert, CloudUpload, CloudOff, Gamepad2, Gauge, Search, TriangleAlert } from "@lucide/svelte";
    import { formatPublished } from "$lib/features/alerts/alerts";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { patchNotesIndexing } from "$lib/features/patch-notes/indexing.svelte";
    import { scanPercent } from "$lib/features/performance/performance";
    import { performanceScan } from "$lib/features/performance/scan.svelte";
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

    const indexing = $derived.by(() => {
        const p = patchNotesIndexing.progress;
        if (!p?.indexing) return null;
        const date = p.currentPublished ? formatPublished(p.currentPublished) : null;
        const percent = p.total > 0 ? Math.min(100, Math.round((p.done / p.total) * 100)) : 0;
        return {
            text: `Indexing patch notes... ${p.done}/${p.total}${date ? ` · ${date}` : ""}`,
            percent,
        };
    });

    const addonScan = $derived.by(() => {
        if (!performanceScan.scanning) return null;
        const total = performanceScan.addons.length;
        return {
            text: `Scanning addons... ${performanceScan.done}/${total}`,
            percent: scanPercent(performanceScan.done, total),
        };
    });
    const performanceIssues = $derived(!performanceScan.scanning && performanceScan.summary.flagged > 0);
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
    {#if indexing}
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
        <span class="flex min-w-0 items-center gap-2 text-muted-foreground/70" title={indexing.text}>
            <Search class="size-3.5 shrink-0" />
            <span class="truncate">{indexing.text}</span>
            <span class="h-1.5 w-20 shrink-0 overflow-hidden rounded-full bg-muted-foreground/20">
                <span
                    class="block h-full rounded-full bg-brass transition-[width] duration-300"
                    style="width: {indexing.percent}%"
                ></span>
            </span>
        </span>
    {/if}
    {#if addonScan}
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
        <span class="flex min-w-0 items-center gap-2 text-muted-foreground/70" title={addonScan.text}>
            <Gauge class="size-3.5 shrink-0" />
            <span class="truncate">{addonScan.text}</span>
            <span class="h-1.5 w-20 shrink-0 overflow-hidden rounded-full bg-muted-foreground/20">
                <span
                    class="block h-full rounded-full bg-brass transition-[width] duration-300"
                    style="width: {addonScan.percent}%"
                ></span>
            </span>
        </span>
    {/if}
    {#if performanceIssues}
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
        <a
            href="/performance"
            class="flex min-w-0 items-center gap-1.5 text-warning hover:underline"
            title="Open Performance"
        >
            <TriangleAlert class="size-3.5 shrink-0" />
            <span class="truncate">Potential performance issues found</span>
        </a>
    {/if}
</footer>
