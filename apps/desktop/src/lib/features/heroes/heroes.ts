export interface Hero {
    id: number;
    name: string;
    icon: string | null;
}

const URL = "https://api.deadlock-api.com/v1/assets/heroes";
const CACHE_KEY = "deadlock-plus:heroes";
const MAX_AGE_MS = 7 * 24 * 60 * 60 * 1000;

interface AssetHero {
    id: number;
    name: string;
    images?: { icon_image_small_webp?: string | null };
}

export function slimHeroes(raw: AssetHero[]): Record<number, Hero> {
    const out: Record<number, Hero> = {};
    for (const h of raw) out[h.id] = { id: h.id, name: h.name, icon: h.images?.icon_image_small_webp ?? null };
    return out;
}

export function parseHeroCache(raw: string | null, now: number, allowStale: boolean): Record<number, Hero> | null {
    if (!raw) return null;
    try {
        const { at, heroes } = JSON.parse(raw) as { at: number; heroes: Record<number, Hero> };
        return allowStale || now - at < MAX_AGE_MS ? heroes : null;
    } catch {
        return null;
    }
}

function readCache(allowStale: boolean): Record<number, Hero> | null {
    try {
        return parseHeroCache(localStorage.getItem(CACHE_KEY), Date.now(), allowStale);
    } catch {
        return null;
    }
}

function writeCache(heroes: Record<number, Hero>) {
    try {
        localStorage.setItem(CACHE_KEY, JSON.stringify({ at: Date.now(), heroes }));
    } catch {
        // Storage can be unavailable; the list is simply fetched again next time.
    }
}

export async function loadHeroes(): Promise<Record<number, Hero>> {
    const cached = readCache(false);
    if (cached) return cached;
    try {
        const res = await fetch(URL);
        if (!res.ok) return readCache(true) ?? {};
        const heroes = slimHeroes((await res.json()) as AssetHero[]);
        writeCache(heroes);
        return heroes;
    } catch {
        return readCache(true) ?? {};
    }
}
