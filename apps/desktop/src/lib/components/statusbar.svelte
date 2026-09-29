<script lang="ts">
    import {
        Activity,
        CircleAlert,
        CloudUpload,
        CloudOff,
        Gamepad2,
        Gauge,
        Search,
        TriangleAlert,
    } from "@lucide/svelte";
    import { ingestStatus } from "$lib/features/ingest/status.svelte";
    import { jobPercent, jobStatusText } from "$lib/features/jobs/jobs";
    import { jobs } from "$lib/features/jobs/jobs.svelte";
    import { performanceScan } from "$lib/features/performance/scan.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";

    const JOB_ICONS: Record<string, typeof Search> = { "patch-notes-index": Search, "addon-scan": Gauge };

    const gameRunning = $derived(jobs.gameRunning);

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
    {#each jobs.active as job (job.id)}
        {@const JobIcon = JOB_ICONS[job.id] ?? Activity}
        {@const text = jobStatusText(job)}
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
        <span class="flex min-w-0 items-center gap-2 text-muted-foreground/70" title={text}>
            <JobIcon class="size-3.5 shrink-0" />
            <span class="truncate">{text}</span>
            {#if job.state === "paused"}
                <button
                    type="button"
                    class="shrink-0 underline hover:text-foreground"
                    onclick={() => void jobs.forceRun(job.id)}
                >
                    Run now
                </button>
            {:else}
                <span class="h-1.5 w-20 shrink-0 overflow-hidden rounded-full bg-muted-foreground/20">
                    <span
                        class="block h-full rounded-full bg-brass transition-[width] duration-300"
                        style="width: {jobPercent(job)}%"
                    ></span>
                </span>
            {/if}
        </span>
    {/each}
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
