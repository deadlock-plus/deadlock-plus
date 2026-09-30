<script lang="ts">
    import * as Tooltip from "$lib/ui/tooltip";
    import Badge from "$lib/ui/badge.svelte";
    import Flag from "$lib/components/flag.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { ChevronRight, Info, Loader2 } from "@lucide/svelte";
    import type { ServerGroup } from "../types";
    import { pingLabel, pingVariant } from "../ping";

    type Props = {
        group: ServerGroup;
        nested?: boolean;
        expandable?: boolean;
        expanded?: boolean;
        blocked: boolean;
        lockedBy?: string | null;
        blockedMembers?: number;
        memberCount?: number;
        busy: boolean;
        external: boolean;
        externalLabel: string;
        pending: boolean;
        ping: number | null | undefined;
        allSiblingsBlocked?: boolean;
        siblingNames?: string[];
        onToggle: (checked: boolean) => void;
        onExpand?: () => void;
        onBlockSiblings?: () => void;
    };

    let {
        group,
        nested = false,
        expandable = false,
        expanded = false,
        blocked,
        lockedBy = null,
        blockedMembers = 0,
        memberCount = 0,
        busy,
        external,
        externalLabel,
        pending,
        ping,
        allSiblingsBlocked = true,
        siblingNames = [],
        onToggle,
        onExpand,
        onBlockSiblings,
    }: Props = $props();

    const isBlocked = $derived(blocked || external);
    const partial = $derived(!isBlocked && blockedMembers > 0);
</script>

<div
    class="flex items-center gap-3 rounded-lg border px-4 {nested
        ? 'border-border/60 bg-card/50 py-2.5'
        : 'border-border bg-card py-3'}"
>
    {#if expandable}
        <button
            type="button"
            onclick={onExpand}
            aria-label={expanded ? "Collapse relays" : "Expand relays"}
            aria-expanded={expanded}
            class="-ml-1 flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground"
        >
            <ChevronRight class="size-4 transition-transform {expanded ? 'rotate-90' : ''}" />
        </button>
    {:else if !nested}
        <span class="-ml-1 size-5 shrink-0"></span>
    {/if}

    <Flag code={group.countryCode} />

    <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
            <span class="truncate text-sm font-medium">{group.description}</span>
            {#if group.isCluster}
                <Badge variant="secondary">{memberCount} relay group{memberCount === 1 ? "" : "s"}</Badge>
            {:else}
                <Badge variant="outline">{group.relayIps.length} relay{group.relayIps.length === 1 ? "" : "s"}</Badge>
            {/if}
            {#if partial}
                <Badge variant="warning">{blockedMembers}/{memberCount} blocked</Badge>
            {/if}
            {#if external}
                <Badge variant="warning">{externalLabel}</Badge>
            {/if}
            {#if group.routingNote}
                <Tooltip.Provider>
                    <Tooltip.Root delayDuration={150}>
                        <Tooltip.Trigger>
                            <Info class="size-3.5 text-muted-foreground" />
                        </Tooltip.Trigger>
                        <Tooltip.Content>
                            {group.routingNote.note}
                            {#if group.routingNote.unblockableMatches.length > 0}
                                <span class="mt-1 block text-muted-foreground">
                                    {group.routingNote.unblockableMatches.join(", ")} has no blockable relays in Valve's data,
                                    so it can't be blocked here.
                                </span>
                            {/if}
                        </Tooltip.Content>
                    </Tooltip.Root>
                </Tooltip.Provider>
            {/if}
        </div>

        {#if group.routingNote && blocked && !allSiblingsBlocked && onBlockSiblings}
            <button
                type="button"
                onclick={onBlockSiblings}
                class="mt-1 text-left text-xs text-muted-foreground underline decoration-dotted underline-offset-2 hover:text-foreground"
            >
                Also block {siblingNames.join(", ")} to cover more of this route
            </button>
        {/if}
    </div>

    <div class="flex w-20 justify-center">
        <Tooltip.Provider>
            <Tooltip.Root delayDuration={150}>
                <Tooltip.Trigger>
                    <Badge variant={pingVariant(ping)} class="min-w-16 justify-center">{pingLabel(ping, pending)}</Badge
                    >
                </Tooltip.Trigger>
                <Tooltip.Content>
                    {#if ping === null}
                        No reply after several attempts. The region may be unreachable from your network or drop ping
                        traffic.
                    {:else}
                        Direct ping to this {group.isCluster ? "region's closest relay" : "relay"}.
                    {/if}
                </Tooltip.Content>
            </Tooltip.Root>
        </Tooltip.Provider>
    </div>

    <div class="flex w-28 items-center justify-end gap-2">
        {#if busy}
            <Loader2 class="size-4 animate-spin text-muted-foreground" />
        {:else}
            <span class="text-xs leading-none font-medium {isBlocked ? 'text-destructive' : 'text-muted-foreground'}">
                {isBlocked ? "Blocked" : "Open"}
            </span>
            <Tooltip.Provider>
                <Tooltip.Root delayDuration={150}>
                    <Tooltip.Trigger>
                        {#snippet child({ props })}
                            <Switch
                                {...props}
                                checked={isBlocked}
                                disabled={external || lockedBy != null}
                                onCheckedChange={onToggle}
                                aria-label="Block {group.description}"
                                class="data-[state=checked]:bg-destructive"
                            />
                        {/snippet}
                    </Tooltip.Trigger>
                    {#if lockedBy}
                        <Tooltip.Content
                            >Blocked because all of {lockedBy} is blocked. Unblock the region to change this.</Tooltip.Content
                        >
                    {/if}
                </Tooltip.Root>
            </Tooltip.Provider>
        {/if}
    </div>
</div>
