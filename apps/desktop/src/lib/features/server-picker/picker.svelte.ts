import { toast } from "svelte-sonner";

import { errorText } from "$lib/core/errors";
import { isGameRunning } from "$lib/features/voice-bans/api";
import {
    blockServerGroups,
    detectExternalBlocks,
    fetchServerGroups,
    firewallCapability,
    getGameDefinitions,
    importExternalBlocks,
    listBlockedGroupIds,
    pingServerGroups,
    syncServerBlocks,
    unblockServerGroups,
} from "./api";
import { readCachedServerData, writeCachedServerData } from "./cache";
import { diffBlocks, readPresets, resolveBlockedIds, writePresets, type Preset } from "./presets";
import {
    allSiblingsBlocked,
    chunk,
    groupPing,
    indexById,
    matchesSearch,
    membersOf,
    siblingNames,
    siblingsToBlock,
} from "./regions";
import { DEFAULT_SORT, nextSort, sortItems, type SortKey, type SortState } from "./sort";
import { readSort, writeSort } from "./sort-store";
import type { ExternalScan, FirewallCapability, GameDefinition, PingResults, ServerData, ServerGroup } from "./types";

const PING_BATCH_SIZE = 8;

const toRule = (g: ServerGroup) => ({ id: g.id, description: g.description, relayIps: g.relayIps });

export class ServerPicker {
    gameDef = $state<GameDefinition | null>(null);
    serverData = $state<ServerData | null>(null);
    capability = $state<FirewallCapability | null>(null);
    search = $state("");
    loading = $state(true);
    pinging = $state(false);
    error = $state<string | null>(null);
    blockedIds = $state<Set<string>>(new Set());
    busyIds = $state<Set<string>>(new Set());
    expandedIds = $state<Set<string>>(new Set());
    externalBlocks = $state<ExternalScan | null>(null);
    importing = $state(false);
    pings = $state<PingResults>({});
    presets = $state<Preset[]>([]);
    presetsOpen = $state(false);
    unblockAllOpen = $state(false);
    importOpen = $state(false);
    sort = $state<SortState>(DEFAULT_SORT);
    gameRunning = $state(false);

    unclusteredById = $derived(indexById(this.serverData?.unclustered ?? []));
    regions = $derived(this.serverData?.clustered ?? []);
    externalIds = $derived(
        new Set(this.externalBlocks?.coveredGroupIds.filter((id) => !this.blockedIds.has(id)) ?? []),
    );
    externalLabel = $derived(this.externalBlocks?.sources.join(" / ") || "Other tool");
    blockedRegionIds = $derived(this.regions.filter((g) => this.blockedIds.has(g.id)).map((g) => g.id));

    visibleRegions = $derived(
        sortItems(
            this.regions.filter((g) => this.matches(g) || this.members(g).some((m) => this.matches(m))),
            this.sort,
            {
                name: (g) => g.description,
                ping: (g) => this.pingOf(g),
                blocked: (g) => this.blockedIds.has(g.id) || this.externalIds.has(g.id),
            },
        ),
    );

    members(group: ServerGroup): ServerGroup[] {
        return membersOf(group, this.unclusteredById);
    }

    matches(group: ServerGroup): boolean {
        return matchesSearch(group, this.search);
    }

    pingOf(group: ServerGroup): number | null | undefined {
        return groupPing(group, group.isCluster ? this.members(group) : [], this.pings);
    }

    visibleMembers(group: ServerGroup): ServerGroup[] {
        const members = this.members(group);
        const shown = this.search && !this.matches(group) ? members.filter((m) => this.matches(m)) : members;
        const parentBlocked = this.blockedIds.has(group.id);
        return sortItems(shown, this.sort, {
            name: (g) => g.description,
            ping: (g) => this.pings[g.id],
            blocked: (g) => parentBlocked || this.blockedIds.has(g.id),
        });
    }

    isExpanded(group: ServerGroup): boolean {
        return this.expandedIds.has(group.id) || (this.search !== "" && !this.matches(group));
    }

    toggleExpanded(id: string) {
        const next = new Set(this.expandedIds);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        this.expandedIds = next;
    }

    sortBy(key: SortKey) {
        this.sort = nextSort(this.sort, key);
        void writeSort(this.sort);
    }

    allSiblingsBlocked(group: ServerGroup): boolean {
        return allSiblingsBlocked(group, this.blockedIds, this.externalIds);
    }

    siblingNames(group: ServerGroup): string[] {
        return siblingNames(group, this.regions, this.blockedIds, this.externalIds);
    }

    init() {
        void this.loadAll();
        void readPresets().then((p) => (this.presets = p));
        void readSort().then((s) => (this.sort = s));
    }

    async refreshGameRunning() {
        try {
            this.gameRunning = await isGameRunning();
        } catch {
            // Keep the last known state; the next poll retries.
        }
    }

    async loadAll() {
        this.loading = true;
        this.error = null;

        const capability = await firewallCapability().catch(() => ({ supported: false }));
        this.capability = capability;

        const defs = await getGameDefinitions().catch(() => []);
        const gameDef = defs[0] ?? null;
        this.gameDef = gameDef;
        if (!gameDef) {
            this.error = "No games are configured yet.";
            this.loading = false;
            return;
        }

        const cached = await readCachedServerData(gameDef.id);
        if (cached) {
            this.serverData = cached;
            this.loading = false;
        }

        try {
            const fresh = await fetchServerGroups(gameDef.id);
            this.serverData = fresh;
            void writeCachedServerData(gameDef.id, fresh);
        } catch (e) {
            if (!cached) {
                this.error = errorText(e);
            } else {
                toast.error("Couldn't refresh the server list, showing cached data.");
            }
        }

        this.loading = false;
        void this.pingAll();

        const serverData = this.serverData;
        if (serverData && capability.supported) {
            await this.syncBlocks();

            try {
                const ids = [...new Set([...serverData.clustered, ...serverData.unclustered].map((g) => g.id))];
                this.blockedIds = new Set(await listBlockedGroupIds(ids));
            } catch {
                toast.error("Couldn't read current firewall rules. Try running as administrator.");
            }

            try {
                this.externalBlocks = await detectExternalBlocks(gameDef.id);
            } catch {
                this.externalBlocks = null;
            }
        }
    }

    private async syncBlocks() {
        try {
            const { updated, failed } = await syncServerBlocks();
            if (updated.length > 0) toast.info(`Valve moved its relays. Updated blocks for ${updated.join(", ")}.`);
            if (failed.length > 0) toast.error(`Couldn't update blocks for ${failed.join(", ")}. Re-apply them.`);
        } catch (e) {
            toast.error(`Couldn't check your blocks against Valve's relays: ${errorText(e)}`);
        }
    }

    async pingAll() {
        if (!this.serverData || this.pinging) return;
        const groups = this.serverData.unclustered;
        if (groups.length === 0) return;

        this.pinging = true;
        this.pings = {};
        try {
            await Promise.all(
                chunk(groups, PING_BATCH_SIZE).map(async (batch) => {
                    try {
                        const results = await pingServerGroups(batch.map((g) => ({ id: g.id, relayIps: g.relayIps })));
                        this.pings = { ...this.pings, ...results };
                    } catch {
                        this.pings = { ...this.pings, ...Object.fromEntries(batch.map((g) => [g.id, null])) };
                    }
                }),
            );
        } finally {
            this.pinging = false;
        }
    }

    private markBusy(ids: string[], busy: boolean) {
        const next = new Set(this.busyIds);
        for (const id of ids) {
            if (busy) next.add(id);
            else next.delete(id);
        }
        this.busyIds = next;
    }

    async toggleGroup(group: ServerGroup, checked: boolean) {
        if (!this.capability?.supported) {
            toast.error("Blocking servers isn't supported on this platform yet.");
            return;
        }

        this.markBusy([group.id], true);
        try {
            if (checked) {
                await blockServerGroups([toRule(group)]);
                this.blockedIds = new Set(this.blockedIds).add(group.id);
                toast.success(`Blocked ${group.description}`);
            } else {
                await unblockServerGroups([group.id]);
                const next = new Set(this.blockedIds);
                next.delete(group.id);
                this.blockedIds = next;
                toast.success(`Unblocked ${group.description}`);
            }
        } catch (e) {
            toast.error(`Failed to ${checked ? "block" : "unblock"} ${group.description}: ${errorText(e)}`);
        } finally {
            this.markBusy([group.id], false);
        }
    }

    async blockSiblings(group: ServerGroup) {
        if (!this.serverData || !group.routingNote || !this.capability?.supported) return;

        const siblings = siblingsToBlock(group, this.regions, this.blockedIds);
        if (siblings.length === 0) return;

        const ids = siblings.map((g) => g.id);
        this.markBusy(ids, true);
        try {
            await blockServerGroups(siblings.map(toRule));
            this.blockedIds = new Set([...this.blockedIds, ...ids]);
            toast.success(`Blocked ${siblings.map((g) => g.description).join(", ")}`);
        } catch (e) {
            toast.error(`Failed to block related relays: ${errorText(e)}`);
        } finally {
            this.markBusy(ids, false);
        }
    }

    async unblockAll() {
        const ids = [...this.blockedIds];
        if (ids.length === 0) return;

        try {
            await unblockServerGroups(ids);
            this.blockedIds = new Set();
            toast.success(`Unblocked ${ids.length} rule${ids.length === 1 ? "" : "s"}`);
        } catch (e) {
            toast.error(`Failed to unblock everything: ${errorText(e)}`);
        }
    }

    async applyPreset(preset: Preset) {
        if (!this.capability?.supported) {
            toast.error("Blocking servers isn't supported on this platform yet.");
            return;
        }

        const target = resolveBlockedIds(
            preset,
            this.regions.map((g) => g.id),
        );
        const { toBlock, toUnblock } = diffBlocks(target, this.blockedIds);
        const touched = [...toBlock, ...toUnblock];
        this.markBusy(touched, true);
        try {
            if (toBlock.length > 0) {
                const groups = this.regions.filter((g) => toBlock.includes(g.id));
                await blockServerGroups(groups.map(toRule));
            }
            if (toUnblock.length > 0) await unblockServerGroups(toUnblock);

            this.blockedIds = new Set(target);
            this.presetsOpen = false;
            toast.success(`Applied "${preset.name}": ${target.length} of ${this.regions.length} regions blocked`);
        } catch (e) {
            toast.error(`Couldn't apply "${preset.name}": ${errorText(e)}`);
            try {
                const ids = [...new Set([...this.regions, ...(this.serverData?.unclustered ?? [])].map((g) => g.id))];
                this.blockedIds = new Set(await listBlockedGroupIds(ids));
            } catch {
                // Leave the displayed state as-is; the next load re-reads the firewall.
            }
        } finally {
            this.markBusy(touched, false);
        }
    }

    async savePreset(preset: Preset) {
        const next = this.presets.some((p) => p.id === preset.id)
            ? this.presets.map((p) => (p.id === preset.id ? preset : p))
            : [...this.presets, preset];
        try {
            await writePresets(next);
            this.presets = next;
            toast.success(`Saved "${preset.name}"`);
        } catch (e) {
            toast.error(`Couldn't save the preset: ${errorText(e)}`);
        }
    }

    async deletePreset(id: string) {
        const next = this.presets.filter((p) => p.id !== id);
        try {
            await writePresets(next);
            this.presets = next;
        } catch (e) {
            toast.error(`Couldn't delete the preset: ${errorText(e)}`);
        }
    }

    async importExternal() {
        if (!this.gameDef) return;
        this.importing = true;
        try {
            const ids = await importExternalBlocks(this.gameDef.id);
            this.blockedIds = new Set([...this.blockedIds, ...ids]);
            this.externalBlocks = await detectExternalBlocks(this.gameDef.id);
            toast.success(`Imported ${ids.length} block${ids.length === 1 ? "" : "s"} from ${this.externalLabel}`);
        } catch (e) {
            toast.error(`Import failed: ${errorText(e)}`);
        } finally {
            this.importing = false;
        }
    }
}
