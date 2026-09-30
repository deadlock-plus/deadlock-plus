<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { ArrowDown, ArrowUp, Layers, RefreshCw, Search, ShieldOff, TriangleAlert } from "@lucide/svelte";

    import Button from "$lib/components/ui/button.svelte";
    import Input from "$lib/components/ui/input.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";

    import ServerRow from "$lib/features/server-picker/components/server-row.svelte";
    import PresetsDialog from "$lib/features/server-picker/components/presets-dialog.svelte";
    import {
        blockServerGroups,
        detectExternalBlocks,
        importExternalBlocks,
        fetchServerGroups,
        firewallCapability,
        getGameDefinitions,
        listBlockedGroupIds,
        pingServerGroups,
        syncServerBlocks,
        unblockServerGroups,
    } from "$lib/features/server-picker/api";
    import { isGameRunning } from "$lib/features/voice-bans/api";
    import { readCachedServerData, writeCachedServerData } from "$lib/features/server-picker/cache";
    import { bestPing } from "$lib/features/server-picker/estimate";
    import {
        diffBlocks,
        readPresets,
        resolveBlockedIds,
        writePresets,
        type Preset,
    } from "$lib/features/server-picker/presets";
    import { DEFAULT_SORT, nextSort, sortItems, type SortKey, type SortState } from "$lib/features/server-picker/sort";
    import { readSort, writeSort } from "$lib/features/server-picker/sort-store";
    import { platform, platformName } from "$lib/core/platform";
    import type {
        ExternalScan,
        FirewallCapability,
        GameDefinition,
        PingResults,
        ServerData,
        ServerGroup,
    } from "$lib/features/server-picker/types";

    const PING_BATCH_SIZE = 8;
    const GAME_POLL_MS = 3000;

    let gameDef = $state<GameDefinition | null>(null);
    let serverData = $state<ServerData | null>(null);
    let capability = $state<FirewallCapability | null>(null);
    let search = $state("");
    let loading = $state(true);
    let pinging = $state(false);
    let error = $state<string | null>(null);
    let blockedIds = $state<Set<string>>(new Set());
    let busyIds = $state<Set<string>>(new Set());
    let expandedIds = $state<Set<string>>(new Set());
    let externalBlocks = $state<ExternalScan | null>(null);
    let importing = $state(false);
    let pings = $state<PingResults>({});
    let presets = $state<Preset[]>([]);
    let presetsOpen = $state(false);
    let sort = $state<SortState>(DEFAULT_SORT);
    let gameRunning = $state(false);

    const unclusteredById = $derived(new Map((serverData?.unclustered ?? []).map((g) => [g.id, g])));
    const regions = $derived(serverData?.clustered ?? []);

    function membersOf(group: ServerGroup): ServerGroup[] {
        return (group.memberIds ?? [])
            .map((id) => unclusteredById.get(id))
            .filter((g): g is ServerGroup => g !== undefined)
            .sort((a, b) => a.description.localeCompare(b.description));
    }

    function matches(group: ServerGroup): boolean {
        return group.description.toLowerCase().includes(search.toLowerCase());
    }

    const visibleRegions = $derived(
        sortItems(
            regions.filter((g) => matches(g) || membersOf(g).some(matches)),
            sort,
            {
                name: (g) => g.description,
                ping: groupPing,
                blocked: (g) => blockedIds.has(g.id) || externalIds.has(g.id),
            },
        ),
    );

    function visibleMembers(group: ServerGroup): ServerGroup[] {
        const members = membersOf(group);
        const shown = search && !matches(group) ? members.filter(matches) : members;
        const parentBlocked = blockedIds.has(group.id);
        return sortItems(shown, sort, {
            name: (g) => g.description,
            ping: (g) => pings[g.id],
            blocked: (g) => parentBlocked || blockedIds.has(g.id),
        });
    }

    function sortBy(key: SortKey) {
        sort = nextSort(sort, key);
        void writeSort(sort);
    }

    function isExpanded(group: ServerGroup): boolean {
        return expandedIds.has(group.id) || (search !== "" && !matches(group));
    }

    function toggleExpanded(id: string) {
        const next = new Set(expandedIds);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        expandedIds = next;
    }

    function groupPing(group: ServerGroup): number | null | undefined {
        if (!group.isCluster) return pings[group.id];
        const values = membersOf(group).map((m) => pings[m.id]);
        const best = bestPing(values);
        if (best != null) return best;
        return values.length > 0 && values.every((v) => v === null) ? null : undefined;
    }

    async function loadAll() {
        loading = true;
        error = null;

        capability = await firewallCapability().catch(() => ({ supported: false }));

        const defs = await getGameDefinitions().catch(() => []);
        gameDef = defs[0] ?? null;
        if (!gameDef) {
            error = "No games are configured yet.";
            loading = false;
            return;
        }

        const cached = await readCachedServerData(gameDef.id);
        if (cached) {
            serverData = cached;
            loading = false;
        }

        try {
            const fresh = await fetchServerGroups(gameDef.id);
            serverData = fresh;
            void writeCachedServerData(gameDef.id, fresh);
        } catch (e) {
            if (!cached) {
                error = String(e);
            } else {
                toast.error("Couldn't refresh the server list, showing cached data.");
            }
        }

        loading = false;
        void pingAll();

        if (serverData && capability.supported) {
            await syncBlocks();

            try {
                const ids = [...new Set([...serverData.clustered, ...serverData.unclustered].map((g) => g.id))];
                blockedIds = new Set(await listBlockedGroupIds(ids));
            } catch {
                toast.error("Couldn't read current firewall rules. Try running as administrator.");
            }

            try {
                externalBlocks = await detectExternalBlocks(gameDef.id);
            } catch {
                externalBlocks = null;
            }
        }
    }

    async function syncBlocks() {
        try {
            const { updated, failed } = await syncServerBlocks();
            if (updated.length > 0) toast.info(`Valve moved its relays. Updated blocks for ${updated.join(", ")}.`);
            if (failed.length > 0) toast.error(`Couldn't update blocks for ${failed.join(", ")}. Re-apply them.`);
        } catch (e) {
            toast.error(`Couldn't check your blocks against Valve's relays: ${e}`);
        }
    }

    async function pingAll() {
        if (!serverData || pinging) return;
        const groups = serverData.unclustered;
        if (groups.length === 0) return;

        pinging = true;
        pings = {};
        try {
            const batches: ServerGroup[][] = [];
            for (let i = 0; i < groups.length; i += PING_BATCH_SIZE) batches.push(groups.slice(i, i + PING_BATCH_SIZE));

            await Promise.all(
                batches.map(async (batch) => {
                    try {
                        const results = await pingServerGroups(batch.map((g) => ({ id: g.id, relayIps: g.relayIps })));
                        pings = { ...pings, ...results };
                    } catch {
                        pings = { ...pings, ...Object.fromEntries(batch.map((g) => [g.id, null])) };
                    }
                }),
            );
        } finally {
            pinging = false;
        }
    }

    function markBusy(ids: string[], busy: boolean) {
        const next = new Set(busyIds);
        for (const id of ids) {
            if (busy) next.add(id);
            else next.delete(id);
        }
        busyIds = next;
    }

    async function toggleGroup(group: ServerGroup, checked: boolean) {
        if (!capability?.supported) {
            toast.error("Blocking servers isn't supported on this platform yet.");
            return;
        }

        markBusy([group.id], true);
        try {
            if (checked) {
                await blockServerGroups([{ id: group.id, description: group.description, relayIps: group.relayIps }]);
                blockedIds = new Set(blockedIds).add(group.id);
                toast.success(`Blocked ${group.description}`);
            } else {
                await unblockServerGroups([group.id]);
                const next = new Set(blockedIds);
                next.delete(group.id);
                blockedIds = next;
                toast.success(`Unblocked ${group.description}`);
            }
        } catch (e) {
            toast.error(`Failed to ${checked ? "block" : "unblock"} ${group.description}: ${e}`);
        } finally {
            markBusy([group.id], false);
        }
    }

    async function blockSiblings(group: ServerGroup) {
        if (!serverData || !group.routingNote || !capability?.supported) return;

        const siblings = regions.filter(
            (g) => group.routingNote!.relatedGroupIds.includes(g.id) && !blockedIds.has(g.id),
        );
        if (siblings.length === 0) return;

        const ids = siblings.map((g) => g.id);
        markBusy(ids, true);
        try {
            await blockServerGroups(
                siblings.map((g) => ({ id: g.id, description: g.description, relayIps: g.relayIps })),
            );
            blockedIds = new Set([...blockedIds, ...ids]);
            toast.success(`Blocked ${siblings.map((g) => g.description).join(", ")}`);
        } catch (e) {
            toast.error(`Failed to block related relays: ${e}`);
        } finally {
            markBusy(ids, false);
        }
    }

    async function unblockAll() {
        const ids = [...blockedIds];
        if (ids.length === 0) return;

        try {
            await unblockServerGroups(ids);
            blockedIds = new Set();
            toast.success(`Unblocked ${ids.length} rule${ids.length === 1 ? "" : "s"}`);
        } catch (e) {
            toast.error(`Failed to unblock everything: ${e}`);
        }
    }

    async function applyPreset(preset: Preset) {
        if (!capability?.supported) {
            toast.error("Blocking servers isn't supported on this platform yet.");
            return;
        }

        const target = resolveBlockedIds(
            preset,
            regions.map((g) => g.id),
        );
        const { toBlock, toUnblock } = diffBlocks(target, blockedIds);
        const touched = [...toBlock, ...toUnblock];
        markBusy(touched, true);
        try {
            if (toBlock.length > 0) {
                const groups = regions.filter((g) => toBlock.includes(g.id));
                await blockServerGroups(
                    groups.map((g) => ({ id: g.id, description: g.description, relayIps: g.relayIps })),
                );
            }
            if (toUnblock.length > 0) await unblockServerGroups(toUnblock);

            blockedIds = new Set(target);
            presetsOpen = false;
            toast.success(`Applied "${preset.name}": ${target.length} of ${regions.length} regions blocked`);
        } catch (e) {
            toast.error(`Couldn't apply "${preset.name}": ${e}`);
            try {
                const ids = [...new Set([...regions, ...(serverData?.unclustered ?? [])].map((g) => g.id))];
                blockedIds = new Set(await listBlockedGroupIds(ids));
            } catch {
                // Leave the displayed state as-is; the next load re-reads the firewall.
            }
        } finally {
            markBusy(touched, false);
        }
    }

    async function savePreset(preset: Preset) {
        const next = presets.some((p) => p.id === preset.id)
            ? presets.map((p) => (p.id === preset.id ? preset : p))
            : [...presets, preset];
        try {
            await writePresets(next);
            presets = next;
            toast.success(`Saved "${preset.name}"`);
        } catch (e) {
            toast.error(`Couldn't save the preset: ${e}`);
        }
    }

    async function deletePreset(id: string) {
        const next = presets.filter((p) => p.id !== id);
        try {
            await writePresets(next);
            presets = next;
        } catch (e) {
            toast.error(`Couldn't delete the preset: ${e}`);
        }
    }

    const externalIds = $derived(new Set(externalBlocks?.coveredGroupIds.filter((id) => !blockedIds.has(id)) ?? []));
    const externalLabel = $derived(externalBlocks?.sources.join(" / ") || "Other tool");
    const blockedRegionIds = $derived(regions.filter((g) => blockedIds.has(g.id)).map((g) => g.id));

    async function importExternal() {
        if (!gameDef) return;
        importing = true;
        try {
            const ids = await importExternalBlocks(gameDef.id);
            blockedIds = new Set([...blockedIds, ...ids]);
            externalBlocks = await detectExternalBlocks(gameDef.id);
            toast.success(`Imported ${ids.length} block${ids.length === 1 ? "" : "s"} from ${externalLabel}`);
        } catch (e) {
            toast.error(`Import failed: ${e}`);
        } finally {
            importing = false;
        }
    }

    function allSiblingsBlocked(group: ServerGroup): boolean {
        if (!group.routingNote) return true;
        return group.routingNote.relatedGroupIds.every((id) => blockedIds.has(id) || externalIds.has(id));
    }

    function siblingNames(group: ServerGroup): string[] {
        if (!group.routingNote) return [];
        return regions
            .filter(
                (g) =>
                    group.routingNote!.relatedGroupIds.includes(g.id) &&
                    !blockedIds.has(g.id) &&
                    !externalIds.has(g.id),
            )
            .map((g) => g.description);
    }

    async function refreshGameRunning() {
        try {
            gameRunning = await isGameRunning();
        } catch {
            // Keep the last known state; the next poll retries.
        }
    }

    onMount(() => {
        void loadAll();
        void readPresets().then((p) => (presets = p));
        void readSort().then((s) => (sort = s));
        void refreshGameRunning();
        const onFocus = () => void refreshGameRunning();
        window.addEventListener("focus", onFocus);
        const timer = setInterval(() => {
            if (!document.hidden) void refreshGameRunning();
        }, GAME_POLL_MS);
        return () => {
            window.removeEventListener("focus", onFocus);
            clearInterval(timer);
        };
    });
</script>

<div class="mx-auto flex min-h-full max-w-4xl flex-col gap-4 px-6 pb-10 pt-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">{gameDef?.displayName ?? "Deadlock"} Server Picker</h1>
            <p class="text-sm text-muted-foreground">
                Turn a region's toggle on to block it. Blocked regions can't be matched to you, so matchmaking picks
                from the ones left open.
            </p>
        </div>

        <div class="flex shrink-0 items-center gap-2">
            <Button variant="outline" size="sm" onclick={() => (presetsOpen = true)} disabled={loading}>
                <Layers />
                Presets
            </Button>

            <Button variant="outline" size="sm" onclick={pingAll} disabled={loading || pinging}>
                <RefreshCw class={pinging ? "animate-spin" : ""} />
                Ping
            </Button>

            <AlertDialog.Root>
                <AlertDialog.Trigger>
                    {#snippet child({ props })}
                        <Button {...props} variant="outline" size="sm" disabled={blockedIds.size === 0}>
                            <ShieldOff />
                            Unblock all ({blockedIds.size})
                        </Button>
                    {/snippet}
                </AlertDialog.Trigger>
                <AlertDialog.Content>
                    <AlertDialog.Title>Unblock all relays?</AlertDialog.Title>
                    <AlertDialog.Description>
                        This removes all {blockedIds.size} firewall rule{blockedIds.size === 1 ? "" : "s"} Deadlock+ created.
                        Matchmaking will be able to route you to every region again.
                    </AlertDialog.Description>
                    <AlertDialog.Footer>
                        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                        <AlertDialog.Action onclick={unblockAll}>Unblock all</AlertDialog.Action>
                    </AlertDialog.Footer>
                </AlertDialog.Content>
            </AlertDialog.Root>
        </div>
    </header>

    {#if gameRunning}
        <div
            class="flex items-center gap-4 rounded-lg border-2 border-destructive/60 bg-destructive/10 px-5 py-4 text-destructive"
        >
            <TriangleAlert class="size-8 shrink-0" />
            <div>
                <p class="font-heading text-lg font-semibold">Deadlock is running</p>
                <p class="text-sm">
                    Restart your game after changing anything on this page. Blocks only apply to connections made after
                    the game starts.
                </p>
            </div>
        </div>
    {/if}

    {#if capability && !capability.supported}
        <div
            class="flex items-center gap-2 rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning"
        >
            <TriangleAlert class="size-4 shrink-0" />
            Blocking servers isn't available on {platformName(platform)} yet.
        </div>
    {/if}

    {#if externalIds.size > 0}
        <div class="flex items-center justify-between gap-3 rounded-md border border-border bg-card px-3 py-2 text-sm">
            <span>
                Found {externalBlocks?.ruleNames.length}
                {externalLabel} rule{externalBlocks?.ruleNames.length === 1 ? "" : "s"} blocking
                {externalIds.size} region{externalIds.size === 1 ? "" : "s"}.
            </span>
            <AlertDialog.Root>
                <AlertDialog.Trigger>
                    {#snippet child({ props })}
                        <Button {...props} size="sm" disabled={importing}>Import</Button>
                    {/snippet}
                </AlertDialog.Trigger>
                <AlertDialog.Content>
                    <AlertDialog.Title>Import {externalLabel} blocks?</AlertDialog.Title>
                    <AlertDialog.Description>
                        Deadlock+ will recreate these blocks as its own rules, then delete the {externalLabel} rules they
                        replace. Any such rule that blocks something outside these regions is left alone.
                    </AlertDialog.Description>
                    <AlertDialog.Footer>
                        <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                        <AlertDialog.Action onclick={importExternal}>Import</AlertDialog.Action>
                    </AlertDialog.Footer>
                </AlertDialog.Content>
            </AlertDialog.Root>
        </div>
    {/if}

    <div class="relative">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input bind:value={search} placeholder="Filter by region..." aria-label="Filter by region" class="pl-8" />
    </div>

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if loading && !serverData}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Loading relay data...</div>
    {:else}
        <div class="flex items-center gap-3 px-4 text-xs font-medium text-muted-foreground">
            {#snippet sortLabel(key: SortKey, label: string, class_: string)}
                {@const active = sort.key === key}
                <button
                    type="button"
                    onclick={() => sortBy(key)}
                    aria-label="Sort by {label}"
                    class="flex items-center gap-1 rounded hover:text-foreground {active
                        ? 'text-foreground'
                        : ''} {class_}"
                >
                    {label}
                    {#if active}
                        {#if sort.dir === "asc"}
                            <ArrowUp class="size-3" aria-label="ascending" />
                        {:else}
                            <ArrowDown class="size-3" aria-label="descending" />
                        {/if}
                    {/if}
                </button>
            {/snippet}
            <div class="flex-1 pl-8">{@render sortLabel("region", "Region", "")}</div>
            <div class="flex w-20 justify-center">{@render sortLabel("ping", "Direct ping", "whitespace-nowrap")}</div>
            <div class="flex w-28 justify-end">{@render sortLabel("blocked", "Blocked", "")}</div>
        </div>

        <div class="flex flex-col gap-2">
            {#each visibleRegions as group (group.id)}
                {@const members = membersOf(group)}
                {@const expanded = group.isCluster && isExpanded(group)}
                <ServerRow
                    {group}
                    expandable={group.isCluster && members.length > 0}
                    {expanded}
                    memberCount={members.length}
                    blockedMembers={members.filter((m) => blockedIds.has(m.id)).length}
                    blocked={blockedIds.has(group.id)}
                    busy={busyIds.has(group.id)}
                    external={externalIds.has(group.id)}
                    {externalLabel}
                    pending={pinging}
                    ping={groupPing(group)}
                    allSiblingsBlocked={allSiblingsBlocked(group)}
                    siblingNames={siblingNames(group)}
                    onToggle={(checked) => toggleGroup(group, checked)}
                    onExpand={() => toggleExpanded(group.id)}
                    onBlockSiblings={() => blockSiblings(group)}
                />

                {#if expanded}
                    <div class="ml-6 flex flex-col gap-1.5 border-l border-border pl-3">
                        {#each visibleMembers(group) as member (member.id)}
                            <ServerRow
                                group={member}
                                nested
                                blocked={blockedIds.has(member.id) || blockedIds.has(group.id)}
                                lockedBy={blockedIds.has(group.id) ? group.description : null}
                                busy={busyIds.has(member.id)}
                                external={false}
                                {externalLabel}
                                pending={pinging}
                                ping={pings[member.id]}
                                onToggle={(checked) => toggleGroup(member, checked)}
                            />
                        {/each}
                    </div>
                {/if}
            {:else}
                <p class="py-8 text-center text-sm text-muted-foreground">No regions match "{search}".</p>
            {/each}
        </div>
    {/if}
</div>

<PresetsDialog
    bind:open={presetsOpen}
    {presets}
    {regions}
    currentlyBlockedRegionIds={blockedRegionIds}
    onApply={applyPreset}
    onSave={savePreset}
    onDelete={deletePreset}
/>
