<script lang="ts">
    import { toast } from "svelte-sonner";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { ExternalLink } from "@lucide/svelte";

    import Badge from "$lib/components/ui/badge.svelte";
    import Button from "$lib/components/ui/button.svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import { formatPublished, safeExternalUrl, sourceLabel } from "$lib/features/alerts/alerts";
    import {
        contentState,
        getPatchNotes,
        groupByBullet,
        groupBySection,
        groupBySubject,
        renderInline,
        type PatchDetail,
    } from "$lib/features/patch-notes/patch-notes";

    let {
        open = $bindable(false),
        patchId,
        title,
        published,
        origin,
        link,
    }: {
        open?: boolean;
        patchId: string | null;
        title: string;
        published: string;
        origin: string;
        link: string;
    } = $props();

    let detail = $state<PatchDetail | null>(null);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let brokenImages = $state<Set<string>>(new Set());
    let generation = 0;

    const date = $derived(formatPublished(published));
    const viewState = $derived(detail ? contentState(detail.lines, detail.origin) : "empty");
    // A patch with exactly one, never-headed section is prose that never had a real `[ Section ]`
    // header of its own (see `parse::parse_body`'s default) — labelling it "GENERAL" would be noise.
    const sections = $derived(detail && viewState !== "empty" ? groupBySection(detail.lines) : []);
    const showSectionHeadings = $derived(sections.length > 1 || sections[0]?.name !== "General");
    const images = $derived((detail?.images ?? []).filter((src) => !brokenImages.has(src)));

    $effect(() => {
        if (open && patchId) void load(patchId);
        else {
            detail = null;
            error = null;
        }
    });

    async function load(id: string) {
        const mine = ++generation;
        loading = true;
        error = null;
        try {
            const found = await getPatchNotes(id);
            if (mine === generation) detail = found;
        } catch (e) {
            if (mine === generation) error = String(e);
        } finally {
            if (mine === generation) loading = false;
        }
    }

    function openExternal() {
        const url = safeExternalUrl(link);
        if (!url) return toast.error("This update has no valid link.");
        openUrl(url).catch((e) => toast.error(`Could not open the link: ${e}`));
    }

    // The trailing space lives inside the string, not the template: a line-wrapped `<span>...
    // </span>` puts a newline-only text node between the colon and the next value, and Svelte
    // drops that node entirely rather than collapsing it to a space.
    function subjectPrefix(subject: string | null) {
        return subject ? `${subject}: ` : "";
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="h-[80vh] max-w-3xl">
        <div class="flex flex-col gap-1.5 border-b border-border pb-4">
            <div class="flex flex-wrap items-center gap-2">
                <Badge variant="outline">{sourceLabel(origin)}</Badge>
                {#if date}<span class="text-sm text-muted-foreground">{date}</span>{/if}
            </div>
            <Dialog.Title class="font-heading text-2xl font-semibold leading-tight tracking-wide">{title}</Dialog.Title>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto pr-1">
            {#if loading}
                <p class="py-6 text-center text-base text-muted-foreground">Loading full notes...</p>
            {:else if error}
                <p class="py-6 text-center text-base text-destructive">Could not load these notes: {error}</p>
            {:else if viewState === "empty"}
                <p class="py-6 text-center text-base text-muted-foreground">
                    Full notes for this update have not been indexed yet.
                </p>
            {:else}
                {#if viewState === "shallow"}
                    <div
                        class="mb-5 flex flex-wrap items-center justify-between gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2 text-sm text-muted-foreground"
                    >
                        <span>This is the forum's short preview, not the full post.</span>
                        <Button variant="link" class="h-auto p-0" onclick={openExternal}>
                            Read the full post on {sourceLabel(origin)}
                        </Button>
                    </div>
                {/if}
                {#if images.length > 0}
                    <div class="mb-5 flex flex-col gap-3">
                        {#each images as src (src)}
                            <img
                                {src}
                                alt=""
                                loading="lazy"
                                class="max-h-72 w-full rounded-lg border border-border bg-background object-contain"
                                onerror={() => (brokenImages = new Set(brokenImages).add(src))}
                            />
                        {/each}
                    </div>
                {/if}
                <div class="flex flex-col gap-6">
                    {#each sections as section (section.name + section.lines[0].raw)}
                        <section class="flex flex-col gap-3">
                            {#if showSectionHeadings}
                                <h3 class="font-heading text-sm font-semibold uppercase tracking-[0.15em] text-brass">
                                    {section.name}
                                </h3>
                            {/if}
                            {#each groupByBullet(section.lines) as run, r (r)}
                                {#if run.bullet}
                                    <ul class="flex flex-col gap-2.5 border-l-2 border-border pl-4">
                                        {#each groupBySubject(run.lines) as group, i (i)}
                                            <li class="flex gap-2.5 text-base leading-relaxed text-foreground/90">
                                                <span
                                                    class="mt-2.5 size-1 shrink-0 rounded-full bg-brass/70"
                                                    aria-hidden="true"
                                                ></span>
                                                <div class="flex-1">
                                                    {#if group.items.length === 1}
                                                        {#if group.subject}<span class="font-semibold text-foreground"
                                                                >{@html renderInline(
                                                                    subjectPrefix(group.subject),
                                                                )}</span
                                                            >{/if}{@html renderInline(group.items[0])}
                                                    {:else}
                                                        {#if group.subject}<span class="font-semibold text-foreground"
                                                                >{@html renderInline(group.subject)}</span
                                                            >{/if}
                                                        <ul class="mt-1.5 flex flex-col gap-1.5">
                                                            {#each group.items as item, j (j)}
                                                                <li class="flex gap-2">
                                                                    <span
                                                                        class="mt-2 size-1 shrink-0 rounded-full bg-muted-foreground/50"
                                                                        aria-hidden="true"
                                                                    ></span>
                                                                    <span>{@html renderInline(item)}</span>
                                                                </li>
                                                            {/each}
                                                        </ul>
                                                    {/if}
                                                </div>
                                            </li>
                                        {/each}
                                    </ul>
                                {:else}
                                    <div class="flex flex-col gap-3">
                                        {#each run.lines as line, i (i)}
                                            <p class="text-base leading-relaxed text-foreground/90">
                                                {@html renderInline(line.raw)}
                                            </p>
                                        {/each}
                                    </div>
                                {/if}
                            {/each}
                        </section>
                    {/each}
                </div>
            {/if}
        </div>

        <div class="flex justify-end gap-2 border-t border-border pt-4">
            <Button variant="outline" onclick={() => (open = false)}>Close</Button>
            <Button onclick={openExternal}>
                View on {sourceLabel(origin)}
                <ExternalLink class="size-4" />
            </Button>
        </div>
    </Dialog.Content>
</Dialog.Root>
