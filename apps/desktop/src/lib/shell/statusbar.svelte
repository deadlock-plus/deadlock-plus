<script lang="ts">
    import { formatNumber, t } from "$lib/core/i18n.svelte";
    import {
        Activity,
        CircleAlert,
        CloudUpload,
        CloudOff,
        Gamepad2,
        Gauge,
        Search,
        Server,
        ServerOff,
        TriangleAlert,
    } from "@lucide/svelte";
    import {
        apiHealth,
        ingestStatus,
        jobPercent,
        jobStatusText,
        jobs,
        performanceScan,
        settings,
    } from "$lib/features/registry";

    const JOB_ICONS: Record<string, typeof Search> = { "patch-notes-index": Search, "addon-scan": Gauge };

    const gameRunning = $derived(jobs.gameRunning);

    const s = $derived(ingestStatus.status);

    const ingest = $derived.by(() => {
        if (!settings.matchIngest)
            return { icon: CloudOff, tone: "text-muted-foreground/70", text: t("shell.statusbar.ingest.off") };
        if (!s)
            return { icon: CloudUpload, tone: "text-muted-foreground/70", text: t("shell.statusbar.ingest.starting") };
        if (!s.steamFound)
            return { icon: CircleAlert, tone: "text-warning", text: t("shell.statusbar.ingest.no_steam_cache") };
        if (s.lastError)
            return {
                icon: CircleAlert,
                tone: "text-destructive",
                text: t("shell.statusbar.ingest.error", { error: s.lastError }),
            };
        return {
            icon: CloudUpload,
            tone: "text-success",
            text: t("shell.statusbar.ingest.active", { count: formatNumber(s.submitted) }),
        };
    });
    const Icon = $derived(ingest.icon);

    const api = $derived.by(() => {
        const r = apiHealth.result;
        if (!r)
            return {
                icon: Server,
                tone: "text-muted-foreground/70",
                text: t("shell.statusbar.api.checking"),
                title: t("shell.statusbar.api.checking_title"),
            };
        if (r.level === "ok")
            return {
                icon: Server,
                tone: "text-success",
                text: t("shell.statusbar.api.online"),
                title: t("shell.statusbar.api.online_title"),
            };
        if (r.level === "degraded")
            return {
                icon: CircleAlert,
                tone: "text-warning",
                text: t("shell.statusbar.api.degraded"),
                title: t("shell.statusbar.api.degraded_title", { services: r.down.join(", ") }),
            };
        return {
            icon: ServerOff,
            tone: "text-destructive",
            text: t("shell.statusbar.api.offline"),
            title: t("shell.statusbar.api.offline_title"),
        };
    });
    const ApiIcon = $derived(api.icon);

    const performanceIssues = $derived(!performanceScan.scanning && performanceScan.summary.flagged > 0);
</script>

<footer
    class="pointer-events-auto flex h-7 shrink-0 flex-row-reverse items-center gap-4 bg-chrome px-3 text-xs text-muted-foreground select-none"
>
    {#if gameRunning !== null}
        <span
            class="flex items-center gap-1.5 {gameRunning ? 'text-success' : 'text-muted-foreground/70'}"
            title={gameRunning ? t("shell.statusbar.game.running_title") : t("shell.statusbar.game.stopped_title")}
        >
            <Gamepad2 class="size-3.5 shrink-0" />
            <span>{gameRunning ? t("shell.statusbar.game.running") : t("shell.statusbar.game.stopped")}</span>
        </span>
        <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
    {/if}
    <span class="flex items-center gap-1.5 {api.tone}" title={api.title}>
        <ApiIcon class="size-3.5 shrink-0" />
        <span>{api.text}</span>
    </span>
    <span class="text-muted-foreground/30" aria-hidden="true">&middot;</span>
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
                    {t("shell.statusbar.run_now")}
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
            title={t("shell.statusbar.open_performance")}
        >
            <TriangleAlert class="size-3.5 shrink-0" />
            <span class="truncate">{t("shell.statusbar.performance_issues")}</span>
        </a>
    {/if}
</footer>
