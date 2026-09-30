<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { ArrowRight, Eraser, Eye, EyeOff, FolderOpen, LoaderCircle, RefreshCw } from "@lucide/svelte";

    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import * as AlertDialog from "$lib/ui/alert-dialog";
    import { formatBytes } from "$lib/features/demos/demos";
    import {
        clearAllCopy,
        clearCopy,
        describeUnits,
        ENTRY_META,
        groupEntries,
        KIND_META,
        knownTotal,
        reclaimable,
        regenerableIds,
        sizeShare,
        storageClear,
        storageEntries,
        storageEntryStats,
        storageReveal,
        type EntryId,
        type EntryInfo,
        type StatsById,
    } from "$lib/features/storage/storage";

    let entries = $state<EntryInfo[]>([]);
    let stats = $state<StatsById>({});
    let failed = $state<Set<EntryId>>(new Set());
    let loading = $state(true);
    let error = $state<string | null>(null);
    let clearing = $state(false);
    let pendingClear = $state<EntryId | null>(null);
    let pendingAll = $state(false);
    let showPaths = $state(false);
    let now = $state(Math.floor(Date.now() / 1000));

    const total = $derived(knownTotal(stats));
    const groups = $derived(groupEntries(entries, stats));
    const canClear = $derived(reclaimable(entries, stats));
    const regenerable = $derived(regenerableIds(entries, stats));
    const copy = $derived(
        pendingAll
            ? clearAllCopy(regenerable, stats)
            : pendingClear
              ? clearCopy(pendingClear, stats[pendingClear]?.bytes ?? 0)
              : null,
    );

    async function loadStats(id: EntryId) {
        try {
            stats[id] = await storageEntryStats(id);
            failed.delete(id);
        } catch {
            failed.add(id);
        }
    }

    async function load() {
        loading = true;
        error = null;
        try {
            entries = await storageEntries();
        } catch (e) {
            error = String(e);
            loading = false;
            return;
        }
        stats = {};
        failed = new Set();
        now = Math.floor(Date.now() / 1000);
        loading = false;
        await Promise.all(entries.filter((e) => e.path).map((e) => loadStats(e.id)));
    }

    async function reveal(id: EntryId) {
        try {
            await storageReveal(id);
        } catch (e) {
            toast.error(String(e));
        }
    }

    async function confirmClear() {
        const ids = pendingAll ? [...regenerable] : pendingClear ? [pendingClear] : [];
        if (ids.length === 0) return;
        clearing = true;
        let freed = 0;
        let removed = 0;
        const problems: string[] = [];
        for (const id of ids) {
            try {
                const report = await storageClear(id);
                freed += report.freedBytes;
                removed += report.removed;
                problems.push(...report.failed);
            } catch (e) {
                problems.push(String(e));
                break;
            }
        }
        if (removed > 0) toast.success(`Freed ${formatBytes(freed)}`);
        else if (problems.length === 0) toast.info("Nothing to clear.");
        if (problems.length > 0) toast.error(`Couldn't remove ${problems.length}: ${problems[0]}`);
        clearing = false;
        pendingClear = null;
        pendingAll = false;
        for (const id of ids) delete stats[id];
        await Promise.all(ids.map(loadStats));
    }

    onMount(() => void load());
</script>

<Page>
    <PageHeader title="Storage" subtitle="What Deadlock and Deadlock+ keep on this PC.">
        {#snippet actions()}
            <Button variant="outline" size="sm" onclick={() => (showPaths = !showPaths)}>
                {#if showPaths}<EyeOff />Hide paths{:else}<Eye />Show paths{/if}
            </Button>
            <Button variant="outline" size="sm" onclick={load} disabled={loading}>
                <RefreshCw class={loading ? "animate-spin" : ""} />
                Refresh
            </Button>
        {/snippet}
    </PageHeader>

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if loading}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Looking around...</div>
    {:else}
        <Card
            as="section"
            radius="md"
            padding="none"
            class="flex flex-wrap items-center justify-between gap-4 px-5 py-4"
        >
            <div class="flex gap-8">
                <div>
                    <p class="text-xs text-muted-foreground">Total found</p>
                    <p class="text-2xl tabular-nums">{formatBytes(total)}</p>
                </div>
                <div>
                    <p class="text-xs text-muted-foreground">Can be cleared here</p>
                    <p class="text-2xl tabular-nums">{formatBytes(canClear)}</p>
                </div>
            </div>
            <Button
                variant="outline"
                disabled={regenerable.length === 0 || clearing}
                onclick={() => (pendingAll = true)}
            >
                <Eraser />
                Clear regenerable
            </Button>
        </Card>

        {#each groups as group (group.owner.id)}
            <section class="flex flex-col gap-1.5">
                <div class="flex items-baseline justify-between gap-4 px-1">
                    <div class="flex items-baseline gap-2">
                        <h2 class="text-lg">{group.owner.label}</h2>
                        <p class="text-xs text-muted-foreground">{group.owner.blurb}</p>
                    </div>
                    <p class="text-sm tabular-nums text-muted-foreground">{formatBytes(group.bytes)}</p>
                </div>

                <ul class="flex flex-col gap-1.5">
                    {#each group.entries as entry (entry.id)}
                        {@const meta = ENTRY_META[entry.id]}
                        {@const kind = KIND_META[meta.kind]}
                        {@const entryStats = stats[entry.id]}
                        {@const units = entryStats ? describeUnits(entry.id, entryStats, now) : null}
                        <Card as="li" radius="md" padding="row" class="flex items-center gap-3">
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center gap-2">
                                    <p class="text-sm font-semibold text-foreground">{meta.label}</p>
                                    <Badge variant={kind.variant} title={kind.hint}>{kind.label}</Badge>
                                </div>
                                <p class="mt-0.5 text-xs text-foreground/80">{meta.description}</p>
                                <p class="mt-0.5 text-xs text-muted-foreground">{meta.consequence}</p>
                                {#if units}
                                    <p class="mt-0.5 text-xs text-muted-foreground">{units}</p>
                                {/if}
                                {#if entryStats && entryStats.bytes > 0}
                                    <div class="mt-2 h-1 w-full overflow-hidden rounded-full bg-muted">
                                        <div
                                            class="h-full rounded-full bg-primary/70"
                                            style="width: {Math.max(2, sizeShare(entryStats.bytes, total) * 100)}%"
                                        ></div>
                                    </div>
                                {/if}
                                {#if showPaths}
                                    <p
                                        class="mt-1.5 truncate font-mono text-[11px] text-muted-foreground/60"
                                        title={entry.path ?? ""}
                                    >
                                        {entry.path ?? "Not found on this PC"}
                                    </p>
                                {/if}
                            </div>

                            <div class="w-20 shrink-0 text-right text-sm tabular-nums">
                                {#if !entry.path}
                                    <span class="text-xs text-muted-foreground">Not found</span>
                                {:else if failed.has(entry.id)}
                                    <button
                                        type="button"
                                        class="text-xs text-destructive underline"
                                        onclick={() => loadStats(entry.id)}>Retry</button
                                    >
                                {:else if entryStats === undefined}
                                    <LoaderCircle
                                        class="ml-auto size-4 animate-spin text-muted-foreground"
                                        aria-label="Measuring"
                                    />
                                {:else}
                                    {formatBytes(entryStats.bytes)}
                                {/if}
                            </div>

                            <div class="flex shrink-0 items-center gap-1">
                                <div class="flex w-9 justify-center">
                                    {#if meta.link}
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            aria-label="Open {meta.label} page"
                                            title="Open the {meta.label} page"
                                            onclick={() => goto(meta.link!)}
                                        >
                                            <ArrowRight />
                                        </Button>
                                    {/if}
                                </div>
                                <div class="flex w-9 justify-center">
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        aria-label="Show {meta.label} in folder"
                                        title="Show in folder"
                                        disabled={!entry.path}
                                        onclick={() => reveal(entry.id)}
                                    >
                                        <FolderOpen />
                                    </Button>
                                </div>
                                <div class="flex w-9 justify-center">
                                    {#if entry.clearable}
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            aria-label="Clear {meta.label}"
                                            title="Clear"
                                            disabled={!entry.path || clearing || entryStats?.bytes === 0}
                                            onclick={() => (pendingClear = entry.id)}
                                        >
                                            <Eraser />
                                        </Button>
                                    {/if}
                                </div>
                            </div>
                        </Card>
                    {/each}
                </ul>
            </section>
        {/each}

        <p class="text-xs text-muted-foreground">
            Only things the game or Deadlock+ rebuild or keep for you can be cleared here. Everything else belongs to
            the game or your mod manager.
        </p>
    {/if}
</Page>

<AlertDialog.Root
    open={pendingClear !== null || pendingAll}
    onOpenChange={(open) => {
        if (!open && !clearing) {
            pendingClear = null;
            pendingAll = false;
        }
    }}
>
    <AlertDialog.Content class="max-w-md">
        <div class="flex flex-col gap-1.5">
            <AlertDialog.Title>{copy?.title}</AlertDialog.Title>
            <AlertDialog.Description>{copy?.body}</AlertDialog.Description>
        </div>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action variant="destructive" onclick={confirmClear} disabled={clearing}
                >Clear</AlertDialog.Action
            >
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
