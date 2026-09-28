<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { ArrowRight, Eraser, FolderOpen, LoaderCircle, RefreshCw } from "@lucide/svelte";

    import Button from "$lib/components/ui/button.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { formatBytes } from "$lib/features/demos/demos";
    import {
        clearCopy,
        ENTRY_META,
        knownTotal,
        storageClear,
        storageEntries,
        storageEntrySize,
        storageReveal,
        type EntryId,
        type EntryInfo,
    } from "$lib/features/storage/storage";

    let entries = $state<EntryInfo[]>([]);
    let sizes = $state<Partial<Record<EntryId, number>>>({});
    let failed = $state<Set<EntryId>>(new Set());
    let loading = $state(true);
    let error = $state<string | null>(null);
    let clearing = $state(false);
    let pendingClear = $state<EntryId | null>(null);

    const total = $derived(knownTotal(sizes));
    const copy = $derived(pendingClear ? clearCopy(pendingClear, sizes[pendingClear] ?? 0) : null);

    async function loadSize(id: EntryId) {
        try {
            sizes[id] = await storageEntrySize(id);
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
        sizes = {};
        failed = new Set();
        loading = false;
        await Promise.all(entries.filter((e) => e.path).map((e) => loadSize(e.id)));
    }

    async function reveal(id: EntryId) {
        try {
            await storageReveal(id);
        } catch (e) {
            toast.error(String(e));
        }
    }

    async function confirmClear() {
        const id = pendingClear;
        if (!id) return;
        clearing = true;
        try {
            const report = await storageClear(id);
            if (report.removed > 0) toast.success(`Freed ${formatBytes(report.freedBytes)}`);
            else if (report.failed.length === 0) toast.info("Nothing to clear.");
            if (report.failed.length > 0) toast.error(`Couldn't remove ${report.failed.length}: ${report.failed[0]}`);
        } catch (e) {
            toast.error(String(e));
        } finally {
            clearing = false;
            pendingClear = null;
            delete sizes[id];
            await loadSize(id);
        }
    }

    onMount(() => void load());
</script>

<div class="mx-auto flex min-h-full max-w-4xl flex-col gap-4 px-6 pb-10 pt-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">Storage</h1>
            <p class="text-sm text-muted-foreground">What Deadlock and Deadlock+ keep on this PC.</p>
        </div>
        <Button variant="outline" size="sm" onclick={load} disabled={loading}>
            <RefreshCw class={loading ? "animate-spin" : ""} />
            Refresh
        </Button>
    </header>

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if loading}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Looking around...</div>
    {:else}
        <p class="text-sm text-muted-foreground">Total found: {formatBytes(total)}</p>

        <ul class="flex flex-col gap-1.5">
            {#each entries as entry (entry.id)}
                {@const meta = ENTRY_META[entry.id]}
                <li class="flex items-center gap-3 rounded-md border border-border bg-card px-4 py-3">
                    <div class="min-w-0 flex-1">
                        <p class="text-sm font-semibold text-foreground">{meta.label}</p>
                        <p class="mt-0.5 text-xs text-muted-foreground">{meta.description}</p>
                        <p
                            class="mt-1.5 truncate font-mono text-[11px] text-muted-foreground/60"
                            title={entry.path ?? ""}
                        >
                            {entry.path ?? "Not found on this PC"}
                        </p>
                    </div>

                    <div class="w-20 shrink-0 text-right text-sm tabular-nums">
                        {#if !entry.path}
                            <span class="text-muted-foreground">–</span>
                        {:else if failed.has(entry.id)}
                            <button
                                type="button"
                                class="text-xs text-destructive underline"
                                onclick={() => loadSize(entry.id)}>Retry</button
                            >
                        {:else if sizes[entry.id] === undefined}
                            <LoaderCircle
                                class="ml-auto size-4 animate-spin text-muted-foreground"
                                aria-label="Measuring"
                            />
                        {:else}
                            {formatBytes(sizes[entry.id] ?? 0)}
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
                                    disabled={!entry.path || clearing || sizes[entry.id] === 0}
                                    onclick={() => (pendingClear = entry.id)}
                                >
                                    <Eraser />
                                </Button>
                            {/if}
                        </div>
                    </div>
                </li>
            {/each}
        </ul>

        <p class="text-xs text-muted-foreground">
            Only the shader cache, the console log and Deadlock+'s own mute list backups can be cleared here. Everything
            else belongs to the game or your mod manager.
        </p>
    {/if}
</div>

<AlertDialog.Root open={pendingClear !== null} onOpenChange={(open) => !open && !clearing && (pendingClear = null)}>
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
