<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { Eye, EyeOff, RefreshCw } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import { formatBytes } from "$lib/features/demos/demos";
    import {
        clearAllCopy,
        clearCopy,
        groupEntries,
        knownTotal,
        reclaimable,
        regenerableIds,
        storageClear,
        storageEntries,
        storageEntryStats,
        storageReveal,
        type EntryId,
        type EntryInfo,
        type StatsById,
    } from "$lib/features/storage/storage";
    import ClearDialog from "$lib/features/storage/components/clear-dialog.svelte";
    import StorageGroup from "$lib/features/storage/components/storage-group.svelte";
    import StorageSummary from "$lib/features/storage/components/storage-summary.svelte";

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
        <StorageSummary
            {total}
            {canClear}
            disabled={regenerable.length === 0 || clearing}
            onclear={() => (pendingAll = true)}
        />

        {#each groups as group (group.owner.id)}
            <StorageGroup
                {group}
                {stats}
                {failed}
                {total}
                {now}
                {showPaths}
                {clearing}
                onretry={loadStats}
                onopen={(link) => goto(link)}
                onreveal={reveal}
                onclear={(id) => (pendingClear = id)}
            />
        {/each}

        <p class="text-xs text-muted-foreground">
            Only things the game or Deadlock+ rebuild or keep for you can be cleared here. Everything else belongs to
            the game or your mod manager.
        </p>
    {/if}
</Page>

<ClearDialog
    open={pendingClear !== null || pendingAll}
    {copy}
    {clearing}
    onconfirm={confirmClear}
    onclose={() => {
        pendingClear = null;
        pendingAll = false;
    }}
/>
