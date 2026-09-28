<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { ExternalLink, Newspaper, Search, X } from "@lucide/svelte";

    import Badge from "$lib/components/ui/badge.svelte";
    import { alerts } from "$lib/features/alerts/alerts.svelte";
    import { formatPublished, kindTone, sourceLabel } from "$lib/features/alerts/alerts";
    import PatchViewerDialog from "$lib/features/patch-notes/components/patch-viewer-dialog.svelte";
    import { searchPatchNotes, type PatchSearchResult } from "$lib/features/patch-notes/patch-notes";
    import { patchNotesIndexing } from "$lib/features/patch-notes/indexing.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";

    let brokenImages = $state<Set<string>>(new Set());

    let query = $state("");
    let results = $state<PatchSearchResult[]>([]);
    let searching = $state(false);
    let searchToken = 0;

    let viewerOpen = $state(false);
    let viewer = $state<{ patchId: string; title: string; published: string; origin: string; link: string } | null>(
        null,
    );

    const isIndexing = $derived(patchNotesIndexing.progress?.indexing ?? false);

    // Rows keep their unread marker while the page is open; they clear when you leave.
    onMount(() => {
        void alerts.fetchNow();
        return () => void alerts.markAllRead();
    });

    function viewResult(r: PatchSearchResult) {
        viewer = { patchId: r.patchId, title: r.title, published: r.published, origin: r.origin, link: r.link };
        viewerOpen = true;
    }

    function viewAlert(item: (typeof alerts.items)[number]) {
        viewer = {
            patchId: item.id,
            title: item.title,
            published: item.published,
            origin: item.source,
            link: item.link,
        };
        viewerOpen = true;
    }

    let debounce: ReturnType<typeof setTimeout> | undefined;
    function onSearchInput() {
        clearTimeout(debounce);
        if (!query.trim()) {
            results = [];
            searching = false;
            return;
        }
        searching = true;
        debounce = setTimeout(runSearch, 250);
    }

    async function runSearch() {
        const token = ++searchToken;
        const q = query;
        try {
            const found = await searchPatchNotes(q);
            if (token === searchToken) results = found;
        } catch (e) {
            if (token === searchToken) toast.error(`Search failed: ${e}`);
        } finally {
            if (token === searchToken) searching = false;
        }
    }

    function clearSearch() {
        query = "";
        results = [];
        searching = false;
        clearTimeout(debounce);
    }
</script>

<div class="mx-auto flex min-h-full max-w-5xl flex-col gap-5 px-8 pb-12 pt-8">
    <header class="flex flex-col gap-4">
        <div>
            <h1 class="text-3xl">Updates</h1>
            <p class="mt-1 text-base text-muted-foreground">Recent Deadlock patch notes and Steam announcements.</p>
        </div>
        <div class="relative w-full">
            <Search class="pointer-events-none absolute left-4 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
            <input
                type="text"
                bind:value={query}
                oninput={onSearchInput}
                placeholder="Search patch notes... try a hero, an item, or describe the change"
                class="h-14 w-full rounded-xl border border-border bg-card pl-12 pr-12 text-lg outline-none focus-visible:ring-2 focus-visible:ring-brass/50"
            />
            {#if query}
                <button
                    type="button"
                    aria-label="Clear search"
                    class="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    onclick={clearSearch}
                >
                    <X class="size-5" />
                </button>
            {/if}
        </div>
    </header>

    {#if query}
        {#if isIndexing}
            <div class="flex flex-1 items-center justify-center text-base text-muted-foreground">
                Still indexing, please wait...
            </div>
        {:else if searching}
            <div class="flex flex-1 items-center justify-center text-base text-muted-foreground">Searching...</div>
        {:else if results.length === 0}
            <div
                class="flex flex-1 flex-col items-center justify-center gap-1 text-center text-base text-muted-foreground"
            >
                <p>No matches for "{query}".</p>
                <p>Try a hero or item name, or describe the change differently.</p>
            </div>
        {:else}
            <ul class="flex flex-col gap-2">
                {#each results as r (r.patchId + r.snippet)}
                    {@const date = formatPublished(r.published)}
                    <li>
                        <button
                            type="button"
                            class="flex w-full flex-col gap-1.5 rounded-lg border border-border bg-card p-4 text-left transition-colors hover:border-brass/50 hover:bg-accent/40"
                            onclick={() => viewResult(r)}
                        >
                            <div class="flex flex-wrap items-center gap-2">
                                <Badge variant="secondary" class="px-2.5 py-0.5 text-sm">{r.section}</Badge>
                                <Badge variant="outline" class="px-2.5 py-0.5 text-sm">{sourceLabel(r.origin)}</Badge>
                                <span class="text-sm font-medium text-foreground">{r.title}</span>
                                {#if date}<span class="text-sm text-muted-foreground">{date}</span>{/if}
                            </div>
                            <p class="text-base leading-relaxed text-foreground">{r.snippet}</p>
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    {:else if alerts.items.length === 0}
        <div class="flex flex-1 flex-col items-center justify-center gap-1 text-center text-base text-muted-foreground">
            <p>No updates loaded.</p>
            <p>
                Check your connection, or {settings.updateAlerts
                    ? "reopen this page to retry"
                    : "turn on alerts in Settings to be notified of new ones"}.
            </p>
        </div>
    {:else}
        <ul class="flex flex-col gap-4">
            {#each alerts.items as item (item.id)}
                {@const date = formatPublished(item.published)}
                {@const showImage = item.image !== null && !brokenImages.has(item.id)}
                <li>
                    <button
                        type="button"
                        class="group flex w-full flex-col gap-4 rounded-lg border border-border bg-card p-4 text-left transition-colors hover:border-brass/50 hover:bg-accent/40"
                        onclick={() => viewAlert(item)}
                    >
                        <div class="flex items-stretch gap-5">
                            <div
                                class="hidden aspect-video w-72 shrink-0 overflow-hidden rounded-md bg-background sm:block"
                            >
                                {#if showImage}
                                    <img
                                        src={item.image}
                                        alt=""
                                        loading="lazy"
                                        class="size-full object-contain"
                                        onerror={() => (brokenImages = new Set(brokenImages).add(item.id))}
                                    />
                                {:else}
                                    <div class="flex size-full items-center justify-center text-muted-foreground/40">
                                        <Newspaper class="size-10" aria-hidden="true" />
                                    </div>
                                {/if}
                            </div>

                            <div class="flex min-w-0 flex-1 flex-col justify-center gap-3">
                                <div class="flex flex-wrap items-center gap-2">
                                    <Badge
                                        variant={kindTone(item.kind) === "notable" ? "warning" : "secondary"}
                                        class="px-3 py-1 text-base"
                                    >
                                        {item.kind}
                                    </Badge>
                                    <Badge variant="outline" class="px-3 py-1 text-base"
                                        >{sourceLabel(item.source)}</Badge
                                    >
                                    {#if date}<span class="text-base text-muted-foreground">{date}</span>{/if}
                                    {#if !item.read}
                                        <span class="ml-auto flex items-center gap-1.5 text-sm font-medium text-brass">
                                            <span class="size-2 rounded-full bg-brass" aria-hidden="true"></span>New
                                        </span>
                                    {/if}
                                </div>

                                <h2
                                    class="font-heading text-3xl font-semibold leading-tight tracking-wide text-foreground"
                                >
                                    {item.title}
                                </h2>

                                <span class="flex items-center gap-1.5 text-base font-medium text-brass">
                                    Read full notes <ExternalLink class="size-4" />
                                </span>
                            </div>
                        </div>

                        {#if item.summary}
                            <p class="line-clamp-6 whitespace-pre-line text-base leading-relaxed text-muted-foreground">
                                {item.summary}
                            </p>
                        {/if}
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<PatchViewerDialog
    bind:open={viewerOpen}
    patchId={viewer?.patchId ?? null}
    title={viewer?.title ?? ""}
    published={viewer?.published ?? ""}
    origin={viewer?.origin ?? "forum"}
    link={viewer?.link ?? ""}
/>
