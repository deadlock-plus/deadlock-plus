import { bestPing } from "./estimate";
import type { PingResults, ServerGroup } from "./types";

export function indexById(groups: ServerGroup[]): Map<string, ServerGroup> {
    return new Map(groups.map((g) => [g.id, g]));
}

export function membersOf(group: ServerGroup, unclusteredById: Map<string, ServerGroup>): ServerGroup[] {
    return (group.memberIds ?? [])
        .map((id) => unclusteredById.get(id))
        .filter((g): g is ServerGroup => g !== undefined)
        .sort((a, b) => a.description.localeCompare(b.description));
}

export function matchesSearch(group: ServerGroup, search: string): boolean {
    return group.description.toLowerCase().includes(search.toLowerCase());
}

/** `null` means every member failed; `undefined` means still waiting on at least one reply. */
export function groupPing(group: ServerGroup, members: ServerGroup[], pings: PingResults): number | null | undefined {
    if (!group.isCluster) return pings[group.id];
    const values = members.map((m) => pings[m.id]);
    const best = bestPing(values);
    if (best != null) return best;
    return values.length > 0 && values.every((v) => v === null) ? null : undefined;
}

export function chunk<T>(items: T[], size: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += size) batches.push(items.slice(i, i + size));
    return batches;
}

export function allSiblingsBlocked(group: ServerGroup, blocked: Set<string>, external: Set<string>): boolean {
    if (!group.routingNote) return true;
    return group.routingNote.relatedGroupIds.every((id) => blocked.has(id) || external.has(id));
}

export function siblingNames(
    group: ServerGroup,
    regions: ServerGroup[],
    blocked: Set<string>,
    external: Set<string>,
): string[] {
    if (!group.routingNote) return [];
    const related = group.routingNote.relatedGroupIds;
    return regions
        .filter((g) => related.includes(g.id) && !blocked.has(g.id) && !external.has(g.id))
        .map((g) => g.description);
}

export function siblingsToBlock(group: ServerGroup, regions: ServerGroup[], blocked: Set<string>): ServerGroup[] {
    if (!group.routingNote) return [];
    const related = group.routingNote.relatedGroupIds;
    return regions.filter((g) => related.includes(g.id) && !blocked.has(g.id));
}
