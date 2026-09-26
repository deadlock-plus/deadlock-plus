export type SortKey = "region" | "ping" | "blocked";
export type SortDir = "asc" | "desc";
export type SortState = { key: SortKey; dir: SortDir };

export type SortAccessors<T> = {
    name: (item: T) => string;
    ping: (item: T) => number | null | undefined;
    blocked: (item: T) => boolean;
};

export const DEFAULT_SORT: SortState = { key: "region", dir: "asc" };

export function nextSort(current: SortState, key: SortKey): SortState {
    if (current.key !== key) return { key, dir: "asc" };
    return { key, dir: current.dir === "asc" ? "desc" : "asc" };
}

export function sortItems<T>(items: T[], sort: SortState, by: SortAccessors<T>): T[] {
    const sign = sort.dir === "asc" ? 1 : -1;
    const byName = (a: T, b: T) => by.name(a).localeCompare(by.name(b));

    return [...items].sort((a, b) => {
        if (sort.key === "ping") {
            const pa = by.ping(a);
            const pb = by.ping(b);
            // Unanswered rows sort last in both directions so they never crowd out real results.
            if (pa == null || pb == null) {
                if (pa == null && pb == null) return byName(a, b);
                return pa == null ? 1 : -1;
            }
            return (pa - pb) * sign || byName(a, b);
        }
        if (sort.key === "blocked") {
            return (Number(by.blocked(a)) - Number(by.blocked(b))) * sign || byName(a, b);
        }
        return byName(a, b) * sign;
    });
}
