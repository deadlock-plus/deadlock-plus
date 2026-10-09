import { prefs } from "$lib/core/prefs";

export interface Hero {
    id: number;
    name: string;
    icon: string | null;
    /** Hero card PNG, sent to Discord as the large image. */
    portrait: string | null;
    /** Small square PNG icon, sent to Discord. */
    artIcon: string | null;
    /** The hero's own hideout line, such as "Plotting in the Hideout". */
    hideoutLine: string | null;
}

const URL = "https://api.deadlock-api.com/v1/assets/heroes";
const MAX_AGE_MS = 7 * 24 * 60 * 60 * 1000;

interface AssetHero {
    id: number;
    name: string;
    hideout_rich_presence?: string | null;
    images?: {
        icon_image_small_webp?: string | null;
        icon_hero_card?: string | null;
        icon_image_small?: string | null;
    };
}

export function slimHeroes(raw: AssetHero[]): Record<number, Hero> {
    const out: Record<number, Hero> = {};
    for (const h of raw) {
        out[h.id] = {
            id: h.id,
            name: h.name,
            icon: h.images?.icon_image_small_webp ?? null,
            portrait: h.images?.icon_hero_card ?? null,
            artIcon: h.images?.icon_image_small ?? null,
            hideoutLine: h.hideout_rich_presence ?? null,
        };
    }
    return out;
}

export function parseHeroCache(raw: string | null, now: number, allowStale: boolean): Record<number, Hero> | null {
    if (!raw) return null;
    try {
        const { at, heroes } = JSON.parse(raw) as { at: number; heroes: Record<number, Hero> };
        const hasArt = Object.values(heroes).every(
            (h) => h.portrait !== undefined && h.artIcon !== undefined && h.hideoutLine !== undefined,
        );
        return allowStale || (hasArt && now - at < MAX_AGE_MS) ? heroes : null;
    } catch {
        return null;
    }
}

function readCache(allowStale: boolean): Record<number, Hero> | null {
    return parseHeroCache(prefs.getString("heroCache"), Date.now(), allowStale);
}

function writeCache(heroes: Record<number, Hero>) {
    prefs.setString("heroCache", JSON.stringify({ at: Date.now(), heroes }));
}

const RETRY_MS = 60 * 60 * 1000;

export function needsRefresh(
    cached: Record<number, Hero>,
    wanted: Iterable<number>,
    lastAttemptAt: number | null,
    now: number,
): boolean {
    if (lastAttemptAt !== null && now - lastAttemptAt < RETRY_MS) return false;
    for (const id of wanted) if (!(id in cached)) return true;
    return false;
}

let lastRefreshAt: number | null = null;
const refreshListeners = new Set<(heroes: Record<number, Hero>) => void>();

/** Fires when a fresh cache was replaced because it lacked a hero the game reported. */
export function onHeroesRefreshed(fn: (heroes: Record<number, Hero>) => void): () => void {
    refreshListeners.add(fn);
    return () => refreshListeners.delete(fn);
}

/**
 * `wanted` are hero ids seen in the game. A fresh cache that lacks one is replaced, at most once an hour,
 * so a hero released after the cache was written shows up without waiting out its expiry.
 */
export async function loadHeroes(wanted: Iterable<number> = []): Promise<Record<number, Hero>> {
    const cached = readCache(false);
    const forced = cached !== null && needsRefresh(cached, wanted, lastRefreshAt, Date.now());
    if (cached && !forced) return cached;
    if (forced) lastRefreshAt = Date.now();
    try {
        const res = await fetch(URL);
        if (!res.ok) return cached ?? readCache(true) ?? {};
        const heroes = slimHeroes((await res.json()) as AssetHero[]);
        writeCache(heroes);
        if (forced) for (const fn of refreshListeners) fn(heroes);
        return heroes;
    } catch {
        return cached ?? readCache(true) ?? {};
    }
}
