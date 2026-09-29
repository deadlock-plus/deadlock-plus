<script lang="ts">
    import { onMount } from "svelte";
    import { ChevronRight, RefreshCw } from "@lucide/svelte";

    import Badge, { type BadgeVariant } from "$lib/components/ui/badge.svelte";
    import Button from "$lib/components/ui/button.svelte";

    import type { AddonInfo, AddonScan, Severity } from "$lib/features/performance/api";
    import {
        RULE_TITLES,
        addonTitle,
        flattenFindings,
        scanPercent,
        worstSeverity,
    } from "$lib/features/performance/performance";
    import { performanceScan } from "$lib/features/performance/scan.svelte";

    const SEVERITY_BADGE: Record<Severity, BadgeVariant> = {
        high: "destructive",
        medium: "warning",
        low: "secondary",
    };

    const listing = $derived(performanceScan.listing);
    const scans = $derived(performanceScan.scans);
    const failures = $derived(performanceScan.failures);
    const error = $derived(performanceScan.error);
    const scanning = $derived(performanceScan.scanning);
    const addons = $derived(performanceScan.addons);
    const summary = $derived(performanceScan.summary);
    const groups = $derived(performanceScan.groups);
    const percent = $derived(scanPercent(performanceScan.done, addons.length));

    onMount(() => {
        if (!performanceScan.hasRun) void performanceScan.run();
    });
</script>

{#snippet addonMeta(addon: AddonInfo, scan: AddonScan | undefined)}
    <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-medium">{addonTitle(addon, scan)}</p>
        <p class="truncate text-xs text-muted-foreground">
            {addon.fileName}{addon.modId ? ` · mod ${addon.modId}` : ""}
        </p>
    </div>
    {#if addon.enabled === false}<Badge variant="outline">Disabled</Badge>{/if}
{/snippet}

{#snippet tile(label: string, value: number, tone: string)}
    <div class="rounded-md border border-border bg-card px-4 py-3">
        <p class="font-heading text-3xl font-semibold {tone}">{value}</p>
        <p class="text-xs text-muted-foreground">{label}</p>
    </div>
{/snippet}

<div class="mx-auto flex min-h-full max-w-4xl flex-col gap-4 px-6 pb-10 pt-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">Performance</h1>
            <p class="text-sm text-muted-foreground">
                Scans the scripts inside your installed Deadlock addons for patterns that can hurt frametimes, like
                timers that pile up. Nothing is changed or run.
            </p>
        </div>
        <Button variant="outline" size="sm" onclick={() => performanceScan.run()} disabled={scanning}>
            <RefreshCw class={scanning ? "animate-spin" : ""} />
            {scanning ? "Scanning..." : "Scan again"}
        </Button>
    </header>

    <div class="rounded-md border border-border bg-card px-4 py-3 text-sm text-muted-foreground">
        This is a best-effort check on the script text. A finding is a hint to look closer, not proof that a mod slows
        your game. A clean result does not clear a mod either. Only comparing frametimes with the mod on and off shows
        the cause.
    </div>

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if listing && listing.dir === null}
        <div class="flex flex-1 items-center justify-center px-6 text-center text-sm text-muted-foreground">
            Could not find your Deadlock install, so there are no addons to scan.
        </div>
    {:else if listing && addons.length === 0}
        <div class="flex flex-1 items-center justify-center px-6 text-center text-sm text-muted-foreground">
            No addons are installed.
        </div>
    {:else if !listing}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Finding addons...</div>
    {:else}
        {#if scanning}
            <div class="flex flex-col gap-2 rounded-md border border-border bg-card px-4 py-3">
                <div class="flex items-center justify-between gap-3 text-sm">
                    <span>Scanning addons... {performanceScan.done} of {addons.length}</span>
                    <span class="truncate font-mono text-xs text-muted-foreground">{performanceScan.current}</span>
                </div>
                <div class="h-1.5 overflow-hidden rounded-full bg-muted-foreground/20">
                    <div
                        class="h-full rounded-full bg-brass transition-[width] duration-300"
                        style="width: {percent}%"
                    ></div>
                </div>
            </div>
        {/if}

        <div class="grid grid-cols-3 gap-3">
            {@render tile(
                `With findings (${summary.findings} finding${summary.findings === 1 ? "" : "s"})`,
                summary.flagged,
                summary.flagged > 0 ? "text-warning" : "",
            )}
            {@render tile("Scripts, no findings", summary.clean, summary.clean > 0 ? "text-success" : "")}
            {@render tile("No scripts", summary.noScripts, "")}
        </div>

        {#if groups.flagged.length > 0}
            <section class="flex flex-col gap-2">
                <h2 class="px-1 text-sm font-medium text-muted-foreground">Worth a look</h2>
                <ul class="flex flex-col gap-2">
                    {#each groups.flagged as addon (addon.fileName)}
                        {@const scan = scans[addon.fileName]}
                        {@const worst = worstSeverity(scan)}
                        <li class="rounded-md border border-border bg-card">
                            <div class="flex items-center gap-3 px-4 py-3">
                                {@render addonMeta(addon, scan)}
                                {#if worst}
                                    <Badge variant={SEVERITY_BADGE[worst]} class="capitalize">{worst}</Badge>
                                {/if}
                            </div>
                            <ul class="flex flex-col divide-y divide-border border-t border-border">
                                {#each flattenFindings(scan) as f}
                                    <li class="flex flex-col gap-2 px-4 py-3">
                                        <div class="flex items-center gap-2">
                                            <Badge variant={SEVERITY_BADGE[f.severity]} class="capitalize"
                                                >{f.severity}</Badge
                                            >
                                            <span class="text-sm font-medium">{RULE_TITLES[f.rule]}</span>
                                        </div>
                                        <p class="text-sm text-muted-foreground">{f.message}</p>
                                        <pre
                                            class="overflow-x-auto rounded-sm bg-muted px-3 py-2 font-mono text-xs"><code
                                                >{f.snippet}</code
                                            ></pre>
                                        <p class="break-all font-mono text-xs text-muted-foreground">
                                            {f.path}:{f.line} · {f.function}
                                        </p>
                                    </li>
                                {/each}
                            </ul>
                        </li>
                    {/each}
                </ul>
            </section>
        {/if}

        {#if groups.failed.length > 0}
            <section class="flex flex-col gap-2">
                <h2 class="px-1 text-sm font-medium text-muted-foreground">Could not read</h2>
                <ul class="flex flex-col gap-1.5">
                    {#each groups.failed as addon (addon.fileName)}
                        <li class="rounded-md border border-border bg-card px-4 py-3">
                            <div class="flex items-center gap-3">
                                {@render addonMeta(addon, undefined)}
                                <Badge variant="destructive">Error</Badge>
                            </div>
                            <p class="mt-2 text-sm text-destructive">{failures[addon.fileName]}</p>
                        </li>
                    {/each}
                </ul>
            </section>
        {/if}

        {#if groups.clean.length > 0}
            <section class="flex flex-col gap-2">
                <h2 class="px-1 text-sm font-medium text-muted-foreground">Scanned, no findings</h2>
                <ul class="flex flex-col gap-1.5">
                    {#each groups.clean as addon (addon.fileName)}
                        {@const scan = scans[addon.fileName]}
                        <li class="flex items-center gap-3 rounded-md border border-border bg-card px-4 py-3">
                            {@render addonMeta(addon, scan)}
                            <Badge variant="success">{scan.scriptsScanned} scripts</Badge>
                        </li>
                    {/each}
                </ul>
            </section>
        {/if}

        {#if groups.pending.length > 0}
            <p class="px-1 text-sm text-muted-foreground">
                Waiting to scan {groups.pending.length} more addon{groups.pending.length === 1 ? "" : "s"}...
            </p>
        {/if}

        {#if groups.noScripts.length > 0}
            <details class="group rounded-md border border-border bg-card">
                <summary class="flex cursor-pointer list-none items-center gap-3 px-4 py-3 text-sm">
                    <ChevronRight
                        class="size-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-90"
                    />
                    <span class="flex-1 text-muted-foreground">
                        {groups.noScripts.length} addon{groups.noScripts.length === 1 ? " has" : "s have"} no scripts
                    </span>
                </summary>
                <ul class="flex flex-col divide-y divide-border border-t border-border">
                    {#each groups.noScripts as addon (addon.fileName)}
                        <li class="flex items-center gap-3 px-4 py-2">
                            {@render addonMeta(addon, scans[addon.fileName])}
                        </li>
                    {/each}
                </ul>
            </details>
        {/if}
    {/if}
</div>
