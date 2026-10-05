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
