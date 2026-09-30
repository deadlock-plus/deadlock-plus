<script lang="ts">
    import { ChevronRight, RefreshCw } from "@lucide/svelte";

    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import { scanPercent } from "../performance";
    import { performanceScan } from "../scan.svelte";
    import AddonMeta from "./addon-meta.svelte";
    import FlaggedAddon from "./flagged-addon.svelte";
    import SummaryTile from "./summary-tile.svelte";

    const listing = $derived(performanceScan.listing);
    const scans = $derived(performanceScan.scans);
    const failures = $derived(performanceScan.failures);
    const error = $derived(performanceScan.error);
    const scanning = $derived(performanceScan.scanning);
    const paused = $derived(performanceScan.paused);
    const addons = $derived(performanceScan.addons);
    const summary = $derived(performanceScan.summary);
    const groups = $derived(performanceScan.groups);
    const percent = $derived(scanPercent(performanceScan.done, addons.length));
</script>

<div class="flex items-start justify-between gap-4">
    <p class="text-sm text-muted-foreground">
        Scans the scripts inside your installed Deadlock addons for patterns that can hurt frametimes, like timers that
        pile up. Nothing is changed or run.
    </p>
    <Button variant="outline" size="sm" onclick={() => performanceScan.run()} disabled={scanning && !paused}>
        <RefreshCw class={scanning && !paused ? "animate-spin" : ""} />
        {paused ? "Run now" : scanning ? "Scanning..." : "Scan again"}
    </Button>
</div>

<Card radius="md" padding="row" class="text-sm text-muted-foreground">
    This is a best-effort check on the script text. A finding is a hint to look closer, not proof that a mod slows your
    game. A clean result does not clear a mod either. Only comparing frametimes with the mod on and off shows the cause.
</Card>

{#if error}
    <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
{:else if listing && listing.dir === null}
    <EmptyState as="div" layout="fill">
        Could not find your Deadlock install, so there are no addons to scan.
    </EmptyState>
{:else if listing && addons.length === 0}
    <EmptyState as="div" layout="fill">No addons are installed.</EmptyState>
{:else if !listing}
    <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Finding addons...</div>
{:else}
    {#if scanning}
        <Card radius="md" padding="row" class="flex flex-col gap-2">
            <div class="flex items-center justify-between gap-3 text-sm">
                <span>
                    {paused ? "Scan paused while Deadlock runs" : "Scanning addons..."}
                    {performanceScan.done} of {addons.length}
                </span>
                <span class="truncate font-mono text-xs text-muted-foreground">{performanceScan.current}</span>
            </div>
            <div class="h-1.5 overflow-hidden rounded-full bg-muted-foreground/20">
                <div
                    class="h-full rounded-full bg-brass transition-[width] duration-300"
                    style="width: {percent}%"
                ></div>
            </div>
        </Card>
    {/if}

    <div class="grid grid-cols-3 gap-3">
        <SummaryTile
            label="With findings ({summary.findings} finding{summary.findings === 1 ? '' : 's'})"
            value={summary.flagged}
            tone={summary.flagged > 0 ? "text-warning" : ""}
        />
        <SummaryTile
            label="Scripts, no findings"
            value={summary.clean}
            tone={summary.clean > 0 ? "text-success" : ""}
        />
        <SummaryTile label="No scripts" value={summary.noScripts} />
    </div>

    {#if groups.flagged.length > 0}
        <section class="flex flex-col gap-2">
            <h2 class="px-1 text-sm font-medium text-muted-foreground">Worth a look</h2>
            <ul class="flex flex-col gap-2">
                {#each groups.flagged as addon (addon.fileName)}
                    <FlaggedAddon {addon} scan={scans[addon.fileName]} />
                {/each}
            </ul>
        </section>
    {/if}

    {#if groups.failed.length > 0}
        <section class="flex flex-col gap-2">
            <h2 class="px-1 text-sm font-medium text-muted-foreground">Could not read</h2>
            <ul class="flex flex-col gap-1.5">
                {#each groups.failed as addon (addon.fileName)}
                    <Card as="li" radius="md" padding="row">
                        <div class="flex items-center gap-3">
                            <AddonMeta {addon} />
                            <Badge variant="destructive">Error</Badge>
                        </div>
                        <p class="mt-2 text-sm text-destructive">{failures[addon.fileName]}</p>
                    </Card>
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
                    <Card as="li" radius="md" padding="row" class="flex items-center gap-3">
                        <AddonMeta {addon} {scan} />
                        <Badge variant="success">{scan.scriptsScanned} scripts</Badge>
                    </Card>
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
                <ChevronRight class="size-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-90" />
                <span class="flex-1 text-muted-foreground">
                    {groups.noScripts.length} addon{groups.noScripts.length === 1 ? " has" : "s have"} no scripts
                </span>
            </summary>
            <ul class="flex flex-col divide-y divide-border border-t border-border">
                {#each groups.noScripts as addon (addon.fileName)}
                    <li class="flex items-center gap-3 px-4 py-2">
                        <AddonMeta {addon} scan={scans[addon.fileName]} />
                    </li>
                {/each}
            </ul>
        </details>
    {/if}
{/if}
