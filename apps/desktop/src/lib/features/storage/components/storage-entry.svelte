<script lang="ts">
    import { ArrowRight, Eraser, FolderOpen, LoaderCircle } from "@lucide/svelte";

    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import { formatBytes } from "$lib/features/demos/demos";
    import {
        describeUnits,
        ENTRY_META,
        KIND_META,
        sizeShare,
        type EntryInfo,
        type EntryStats,
    } from "$lib/features/storage/storage";

    type Props = {
        entry: EntryInfo;
        stats: EntryStats | undefined;
        failed: boolean;
        total: number;
        now: number;
        showPath: boolean;
        clearing: boolean;
        onretry: () => void;
        onopen: (link: string) => void;
        onreveal: () => void;
        onclear: () => void;
    };

    let { entry, stats, failed, total, now, showPath, clearing, onretry, onopen, onreveal, onclear }: Props = $props();

    const meta = $derived(ENTRY_META[entry.id]);
    const kind = $derived(KIND_META[meta.kind]);
    const units = $derived(stats ? describeUnits(entry.id, stats, now) : null);
</script>

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
        {#if stats && stats.bytes > 0}
            <div class="mt-2 h-1 w-full overflow-hidden rounded-full bg-muted">
                <div
                    class="h-full rounded-full bg-primary/70"
                    style="width: {Math.max(2, sizeShare(stats.bytes, total) * 100)}%"
                ></div>
            </div>
        {/if}
        {#if showPath}
            <p class="mt-1.5 truncate font-mono text-[11px] text-muted-foreground/60" title={entry.path ?? ""}>
                {entry.path ?? "Not found on this PC"}
            </p>
        {/if}
    </div>

    <div class="w-20 shrink-0 text-right text-sm tabular-nums">
        {#if !entry.path}
            <span class="text-xs text-muted-foreground">Not found</span>
        {:else if failed}
            <button
                type="button"
                class="text-xs text-destructive underline"
                aria-label="Retry measuring {meta.label}"
                onclick={onretry}
            >
                Retry
            </button>
        {:else if stats === undefined}
            <LoaderCircle class="ml-auto size-4 animate-spin text-muted-foreground" role="img" aria-label="Measuring" />
        {:else}
            {formatBytes(stats.bytes)}
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
                    onclick={() => onopen(meta.link!)}
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
                onclick={onreveal}
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
                    disabled={!entry.path || clearing || stats?.bytes === 0}
                    onclick={onclear}
                >
                    <Eraser />
                </Button>
            {/if}
        </div>
    </div>
</Card>
