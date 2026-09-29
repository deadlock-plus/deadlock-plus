<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import {
        ChevronLeft,
        ChevronRight,
        CircleHelp,
        ExternalLink,
        FolderOpen,
        Pin,
        Ellipsis,
        Eraser,
        RefreshCw,
        Trash2,
        TriangleAlert,
    } from "@lucide/svelte";

    import Button, { buttonVariants } from "$lib/components/ui/button.svelte";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import CleanupDialog from "$lib/features/demos/components/cleanup-dialog.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import Badge, { type BadgeVariant } from "$lib/components/ui/badge.svelte";

    import {
        countByStatus,
        deleteCopy,
        deleteDemos,
        fetchDemoMetadata,
        formatBytes,
        formatDuration,
        listDemos,
        listPinned,
        localAccountIds,
        matchResult,
        myPlayer,
        openReplaysDir,
        previewDelete,
        revealDemo,
        setPinned,
        statlockerMatchUrl,
        statusInfo,
        totalSize,
        unpinnedNames,
        type Demo,
        type DemoListing,
        type DeleteMode,
        type DeletePreview,
        type DemoStatus,
        type MetaResult,
    } from "$lib/features/demos/demos";
    import { loadHeroes, type Hero } from "$lib/features/demos/heroes";
    import { platform, trashName } from "$lib/platform";

    const PAGE_SIZE = 25;
    const META_CONCURRENCY = 3;
    const BADGE: Record<DemoStatus, BadgeVariant> = {
        complete: "success",
        partial: "destructive",
        outdated: "warning",
        unknown: "secondary",
    };

    let listing = $state<DemoListing | null>(null);
    let error = $state<string | null>(null);
    let loading = $state(true);
    let filter = $state<DemoStatus | "all" | "pinned">("all");
    let pinned = $state<Set<number>>(new Set());
    let cleanupOpen = $state(false);
    let page = $state(0);
    let heroes = $state<Record<number, Hero>>({});
    let meta = $state<Record<number, MetaResult>>({});
    const requested = new Set<number>();
    let selected = $state<Set<string>>(new Set());
    let deleteOpen = $state(false);
    let deleteNames = $state<string[]>([]);
    let preview = $state<DeletePreview | null>(null);
    let deleteMode = $state<DeleteMode>("recycle");
    let deleting = $state(false);

    const demos = $derived(listing?.demos ?? []);
    const counts = $derived(countByStatus(demos));
    const filtered = $derived(
        filter === "all"
            ? demos
            : filter === "pinned"
              ? demos.filter((d) => pinned.has(d.matchId))
              : demos.filter((d) => d.status === filter),
    );
    const pinnedCount = $derived(demos.filter((d) => pinned.has(d.matchId)).length);
    const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / PAGE_SIZE)));
    const visible = $derived(filtered.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE));

    let accountIds = $state<number[]>([]);

    // Partial files are never in the API, so they are not requested.
    $effect(() => {
        const ids = visible.filter((d) => d.status !== "partial" && !requested.has(d.matchId)).map((d) => d.matchId);
        for (const id of ids) requested.add(id);
        void fetchMetadata(ids);
    });

    async function fetchMetadata(ids: number[]) {
        let next = 0;
        const worker = async () => {
            while (next < ids.length) {
                const id = ids[next++];
                try {
                    meta[id] = await fetchDemoMetadata(id);
                } catch (e) {
                    meta[id] = { state: "error", message: String(e) };
                }
                if (meta[id].state === "error") requested.delete(id);
            }
        };
        await Promise.all(Array.from({ length: Math.min(META_CONCURRENCY, ids.length) }, worker));
    }

    async function load() {
        loading = true;
        error = null;
        try {
            listing = await listDemos();
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    const copy = $derived(preview ? deleteCopy(preview) : null);
    const selectable = $derived(visible.filter((d) => !pinned.has(d.matchId)));
    const allVisibleSelected = $derived(selectable.length > 0 && selectable.every((d) => selected.has(d.fileName)));

    function toggle(name: string, on: boolean) {
        const next = new Set(selected);
        if (on) next.add(name);
        else next.delete(name);
        selected = next;
    }

    function toggleVisible(on: boolean) {
        const next = new Set(selected);
        for (const d of selectable) {
            if (on) next.add(d.fileName);
            else next.delete(d.fileName);
        }
        selected = next;
    }

    async function pin(d: Demo, on: boolean) {
        try {
            pinned = new Set(await setPinned(d.matchId, on));
            if (on) toggle(d.fileName, false);
        } catch (e) {
            toast.error(String(e));
        }
    }

    async function askDelete(names: string[]) {
        try {
            preview = await previewDelete(names);
        } catch (e) {
            toast.error(String(e));
            return;
        }
        deleteNames = names;
        deleteMode = deleteCopy(preview).canRecycle ? "recycle" : "permanent";
        deleteOpen = true;
    }

    async function confirmDelete() {
        deleting = true;
        try {
            const report = await deleteDemos(deleteNames, deleteMode);
            const done = report.deleted.length;
            if (done > 0) toast.success(`Deleted ${done} replay${done === 1 ? "" : "s"}`);
            if (report.failed.length > 0)
                toast.error(`Couldn't delete ${report.failed.length}: ${report.failed[0].message}`);
            const gone = new Set(report.deleted);
            selected = new Set([...selected].filter((n) => !gone.has(n)));
        } catch (e) {
            toast.error(String(e));
        } finally {
            deleting = false;
            deleteOpen = false;
            await load();
        }
    }

    async function reveal(d: Demo) {
        try {
            await revealDemo(d);
        } catch (e) {
            toast.error(String(e));
        }
    }

    async function openFolder() {
        try {
            await openReplaysDir();
        } catch (e) {
            toast.error(String(e));
        }
    }

    function setFilter(next: DemoStatus | "all" | "pinned") {
        filter = next;
        page = 0;
    }

    function openStatlocker(id: number) {
        void openUrl(statlockerMatchUrl(id));
    }

    const date = (ms: number) =>
        new Date(ms).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });

    onMount(() => {
        void load();
        void listPinned()
            .then((ids) => (pinned = new Set(ids)))
            .catch(() => {});
        void loadHeroes().then((h) => (heroes = h));
        void localAccountIds()
            .then((ids) => (accountIds = ids))
            .catch(() => {});
    });
</script>

<div class="mx-auto flex min-h-full max-w-4xl flex-col gap-4 px-6 pb-10 pt-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">Replays</h1>
            <p class="text-sm text-muted-foreground">Match replays saved by Deadlock on this PC.</p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <Button variant="outline" size="sm" onclick={openFolder} disabled={!listing?.dir}>
                <FolderOpen />
                Open folder
            </Button>
            <Button variant="outline" size="sm" onclick={() => (cleanupOpen = true)} disabled={!listing?.dir}>
                <Eraser />
                Clean up
            </Button>
            <Button
                variant="outline"
                size="sm"
                disabled={selected.size === 0 || deleting}
                onclick={() =>
                    askDelete(
                        unpinnedNames(
                            demos.filter((d) => selected.has(d.fileName)),
                            pinned,
                        ),
                    )}
            >
                <Trash2 />
                Delete selected ({selected.size})
            </Button>
            <Button variant="outline" size="sm" onclick={load} disabled={loading}>
                <RefreshCw class={loading ? "animate-spin" : ""} />
                Refresh
            </Button>
        </div>
    </header>

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if loading && !listing}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Reading replays...</div>
    {:else if listing && !listing.dir}
        <div class="flex flex-1 items-center justify-center px-6 text-center text-sm text-muted-foreground">
            Couldn't find Deadlock's replays folder. Install Deadlock through Steam and watch or download a replay in
            game.
        </div>
    {:else if listing}
        <div class="flex flex-wrap items-center gap-2">
            <Button size="sm" variant={filter === "all" ? "default" : "outline"} onclick={() => setFilter("all")}>
                All ({demos.length})
            </Button>
            {#each ["complete", "outdated", "partial", "unknown"] as const as s (s)}
                {#if counts[s] > 0}
                    <Button size="sm" variant={filter === s ? "default" : "outline"} onclick={() => setFilter(s)}>
                        {statusInfo(s).label} ({counts[s]})
                    </Button>
                {/if}
            {/each}
            <Button size="sm" variant={filter === "pinned" ? "default" : "outline"} onclick={() => setFilter("pinned")}>
                <Pin />
                Pinned ({pinnedCount})
            </Button>
            <span class="ml-auto text-sm text-muted-foreground">
                {filtered.length} replay{filtered.length === 1 ? "" : "s"}, {formatBytes(totalSize(filtered))}
            </span>
        </div>

        <div class="flex items-center gap-3 px-4 text-xs font-medium text-muted-foreground">
            <input
                type="checkbox"
                aria-label="Select this page"
                checked={allVisibleSelected}
                onchange={(e) => toggleVisible(e.currentTarget.checked)}
            />
            <span>Select this page</span>
        </div>

        <ul class="flex flex-col gap-1.5">
            {#each visible as d (d.fileName)}
                {@const info = statusInfo(d.status)}
                {@const m = meta[d.matchId]}
                {@const summary = m?.state === "ok" ? m.summary : null}
                {@const me = summary ? myPlayer(summary, accountIds) : null}
                {@const result = summary ? matchResult(summary, accountIds) : null}
                {@const hero = me ? heroes[me.heroId] : undefined}
                {@const isPinned = pinned.has(d.matchId)}
                <li class="flex items-center gap-3 rounded-md border border-border bg-card px-4 py-2">
                    <input
                        type="checkbox"
                        aria-label="Select match {d.matchId}"
                        checked={selected.has(d.fileName)}
                        disabled={isPinned}
                        onchange={(e) => toggle(d.fileName, e.currentTarget.checked)}
                    />
                    <div class="flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
                        {#if hero?.icon}
                            <img src={hero.icon} alt={hero.name} class="size-full object-cover" />
                        {:else}
                            <CircleHelp class="size-5 text-muted-foreground" aria-label="Unknown hero" />
                        {/if}
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="text-sm font-medium">
                            {hero?.name ?? (me ? `Hero ${me.heroId}` : `Match ${d.matchId}`)}
                            {#if me}<span class="font-normal text-muted-foreground">
                                    · {me.kills}/{me.deaths}/{me.assists}</span
                                >{/if}
                        </p>
                        <p class="text-xs text-muted-foreground">
                            Match {d.matchId} · {date(summary ? summary.startTime * 1000 : d.modifiedMs)}{summary
                                ? ` · ${formatDuration(summary.durationS)}`
                                : ""} · {formatBytes(d.size)}{d.buildNum ? ` · build ${d.buildNum}` : ""}
                            {#if m?.state === "missing"}
                                · not in the Deadlock API yet{/if}
                            {#if m?.state === "error"}
                                · details unavailable{/if}
                        </p>
                    </div>
                    {#if result}
                        <Badge variant={result === "win" ? "success" : "destructive"}
                            >{result === "win" ? "Win" : "Loss"}</Badge
                        >
                    {/if}
                    <Badge variant={BADGE[d.status]} title={info.hint}>{info.label}</Badge>
                    <Button
                        size="sm"
                        variant="ghost"
                        aria-label={isPinned ? "Unpin replay" : "Pin replay"}
                        aria-pressed={isPinned}
                        title={isPinned
                            ? "Pinned: delete and cleanup skip it. Click to unpin."
                            : "Pin: protect from delete and cleanup"}
                        class={isPinned ? "text-primary" : ""}
                        onclick={() => pin(d, !isPinned)}
                    >
                        <Pin class={isPinned ? "fill-current" : ""} />
                    </Button>
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger
                            class={buttonVariants({ variant: "ghost", size: "sm" })}
                            aria-label="More actions"
                            title="More actions"
                        >
                            <Ellipsis />
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content>
                            {#if d.status !== "partial"}
                                <DropdownMenu.Item onSelect={() => openStatlocker(d.matchId)}>
                                    <ExternalLink />
                                    Open on Statlocker
                                </DropdownMenu.Item>
                            {/if}
                            <DropdownMenu.Item onSelect={() => reveal(d)}>
                                <FolderOpen />
                                Show in folder
                            </DropdownMenu.Item>
                            <DropdownMenu.Item
                                class="text-destructive data-[highlighted]:text-destructive"
                                disabled={deleting || isPinned}
                                onSelect={() => askDelete([d.fileName])}
                            >
                                <Trash2 />
                                {isPinned ? "Delete (unpin first)" : "Delete"}
                            </DropdownMenu.Item>
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                </li>
            {:else}
                <li class="py-8 text-center text-sm text-muted-foreground">
                    {demos.length === 0 ? "No replays saved yet." : "No replays with that status."}
                </li>
            {/each}
        </ul>

        {#if pageCount > 1}
            <div class="flex items-center justify-center gap-3">
                <Button
                    variant="outline"
                    size="sm"
                    disabled={page === 0}
                    onclick={() => page--}
                    aria-label="Previous page"
                >
                    <ChevronLeft />
                </Button>
                <span class="text-sm text-muted-foreground">Page {page + 1} of {pageCount}</span>
                <Button
                    variant="outline"
                    size="sm"
                    disabled={page >= pageCount - 1}
                    onclick={() => page++}
                    aria-label="Next page"
                >
                    <ChevronRight />
                </Button>
            </div>
        {/if}

        {#if listing.referenceBuild}
            <p class="text-xs text-muted-foreground">
                "Older build" compares each replay with the newest build found among your replays (build {listing.referenceBuild}).
            </p>
        {/if}
    {/if}
</div>

<CleanupDialog bind:open={cleanupOpen} onreview={(names) => askDelete(names)} />

<AlertDialog.Root bind:open={deleteOpen}>
    <AlertDialog.Content class="max-w-md">
        <div class="flex flex-col gap-1.5">
            <AlertDialog.Title>{copy?.title}</AlertDialog.Title>
            <AlertDialog.Description>
                {#if preview}Frees {formatBytes(preview.totalBytes)}. Replays aren't backed up in Steam Cloud.{/if}
            </AlertDialog.Description>
        </div>

        {#if copy?.notice}
            <div class="flex items-start gap-3 rounded-md border border-warning/40 bg-warning/10 px-3 py-2.5 text-sm">
                <TriangleAlert class="mt-0.5 size-4 shrink-0 text-warning" />
                <p>{copy.notice}</p>
            </div>
        {:else if copy?.canRecycle}
            <div class="flex flex-col gap-2" role="radiogroup" aria-label="Delete method">
                {#each [{ mode: "recycle", label: `Move to ${trashName(platform)}`, hint: "You can restore it from there." }, { mode: "permanent", label: "Delete permanently", hint: "Frees the space now. Can't be undone." }] as const as option (option.mode)}
                    {@const on = deleteMode === option.mode}
                    <button
                        type="button"
                        role="radio"
                        aria-checked={on}
                        class="flex items-center gap-3 rounded-md border px-3 py-2.5 text-left transition-colors {on
                            ? 'border-primary bg-primary/10'
                            : 'border-border hover:bg-accent/40'}"
                        onclick={() => (deleteMode = option.mode)}
                    >
                        <span
                            class="flex size-4 shrink-0 items-center justify-center rounded-full border {on
                                ? 'border-primary'
                                : 'border-muted-foreground/60'}"
                        >
                            {#if on}<span class="size-2 rounded-full bg-primary"></span>{/if}
                        </span>
                        <span class="flex flex-col">
                            <span class="text-sm font-medium">{option.label}</span>
                            <span class="text-xs text-muted-foreground">{option.hint}</span>
                        </span>
                    </button>
                {/each}
            </div>
        {/if}

        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                variant={deleteMode === "permanent" ? "destructive" : "default"}
                onclick={confirmDelete}
                disabled={deleting}
            >
                {deleteMode === "permanent" ? "Delete permanently" : `Move to ${trashName(platform)}`}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
