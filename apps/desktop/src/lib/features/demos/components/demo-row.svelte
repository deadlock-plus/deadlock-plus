<script lang="ts">
    import { CircleHelp, Ellipsis, ExternalLink, FolderOpen, Pin, Trash2 } from "@lucide/svelte";

    import Badge, { type BadgeVariant } from "$lib/ui/badge.svelte";
    import Button, { buttonVariants } from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import * as DropdownMenu from "$lib/ui/dropdown-menu";
    import {
        formatBytes,
        formatDuration,
        matchResult,
        myPlayer,
        statusInfo,
        type Demo,
        type DemoStatus,
        type MetaResult,
    } from "$lib/features/demos/demos";
    import { demoTitle } from "$lib/features/demos/list";
    import type { Hero } from "$lib/features/heroes/heroes";

    const BADGE: Record<DemoStatus, BadgeVariant> = {
        complete: "success",
        partial: "destructive",
        outdated: "warning",
        unknown: "secondary",
    };

    let {
        demo: d,
        meta: m,
        heroes,
        accountIds,
        pinned: isPinned,
        selected,
        deleting,
        onselect,
        onpin,
        onstatlocker,
        onreveal,
        ondelete,
    }: {
        demo: Demo;
        meta: MetaResult | undefined;
        heroes: Record<number, Hero>;
        accountIds: number[];
        pinned: boolean;
        selected: boolean;
        deleting: boolean;
        onselect: (on: boolean) => void;
        onpin: (on: boolean) => void;
        onstatlocker: () => void;
        onreveal: () => void;
        ondelete: () => void;
    } = $props();

    const info = $derived(statusInfo(d.status));
    const summary = $derived(m?.state === "ok" ? m.summary : null);
    const me = $derived(summary ? myPlayer(summary, accountIds) : null);
    const result = $derived(summary ? matchResult(summary, accountIds) : null);
    const hero = $derived(me ? heroes[me.heroId] : undefined);

    const date = (ms: number) =>
        new Date(ms).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
</script>

<Card as="li" radius="md" padding="none" class="flex items-center gap-3 px-4 py-2">
    <input
        type="checkbox"
        aria-label="Select match {d.matchId}"
        checked={selected}
        disabled={isPinned}
        onchange={(e) => onselect(e.currentTarget.checked)}
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
            {demoTitle(hero?.name, me, d.matchId)}
            {#if me}<span class="font-normal text-muted-foreground"> · {me.kills}/{me.deaths}/{me.assists}</span>{/if}
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
        <Badge variant={result === "win" ? "success" : "destructive"}>{result === "win" ? "Win" : "Loss"}</Badge>
    {/if}
    <Badge variant={BADGE[d.status]} title={info.hint}>{info.label}</Badge>
    <Button
        size="sm"
        variant="ghost"
        aria-label="Pin replay"
        aria-pressed={isPinned}
        title={isPinned
            ? "Pinned: delete and cleanup skip it. Click to unpin."
            : "Pin: protect from delete and cleanup"}
        class={isPinned ? "text-primary" : ""}
        onclick={() => onpin(!isPinned)}
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
                <DropdownMenu.Item onSelect={onstatlocker}>
                    <ExternalLink />
                    Open on Statlocker
                </DropdownMenu.Item>
            {/if}
            <DropdownMenu.Item onSelect={onreveal}>
                <FolderOpen />
                Show in folder
            </DropdownMenu.Item>
            <DropdownMenu.Item
                class="text-destructive data-[highlighted]:text-destructive"
                disabled={deleting || isPinned}
                onSelect={ondelete}
            >
                <Trash2 />
                {isPinned ? "Delete (unpin first)" : "Delete"}
            </DropdownMenu.Item>
        </DropdownMenu.Content>
    </DropdownMenu.Root>
</Card>
