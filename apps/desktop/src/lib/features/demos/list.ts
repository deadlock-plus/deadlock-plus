import type { Demo, DemoStatus } from "./demos";

export type { Demo };

export type DemoFilter = DemoStatus | "all" | "pinned";

export const PAGE_SIZE = 25;
export const META_CONCURRENCY = 3;

export function filterDemos(demos: Demo[], filter: DemoFilter, pinned: Set<number>): Demo[] {
    if (filter === "all") return demos;
    if (filter === "pinned") return demos.filter((d) => pinned.has(d.matchId));
    return demos.filter((d) => d.status === filter);
}

export function pinnedCount(demos: Demo[], pinned: Set<number>): number {
    return demos.filter((d) => pinned.has(d.matchId)).length;
}

export function pageCount(total: number, size: number): number {
    return Math.max(1, Math.ceil(total / size));
}

export function pageSlice<T>(items: T[], page: number, size: number): T[] {
    return items.slice(page * size, (page + 1) * size);
}

export function demoTitle(heroName: string | undefined, me: { heroId: number } | null, matchId: number): string {
    return heroName ?? (me ? `Hero ${me.heroId}` : `Match ${matchId}`);
}

export async function runPool<T>(items: T[], concurrency: number, task: (item: T) => Promise<void>): Promise<void> {
    let next = 0;
    const worker = async () => {
        while (next < items.length) await task(items[next++]);
    };
    await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, worker));
}
