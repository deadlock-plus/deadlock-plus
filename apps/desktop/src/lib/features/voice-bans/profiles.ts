import { t } from "$lib/core/i18n.svelte";
import { steam32ToSteam64, steam64ToSteam32 } from "./voice-ban";

const API = "https://api.deadlock-api.com/v1/players";
const BATCH = 50;
const CONCURRENCY = 4;

export interface Profile {
    steamid64: string;
    name: string;
    avatar: string | null;
    profileUrl: string | null;
}

interface ApiPlayer {
    account_id: number;
    personaname?: string | null;
    avatarmedium?: string | null;
    profileurl?: string | null;
}

export function toProfile(p: ApiPlayer): Profile | null {
    const steamid64 = steam32ToSteam64(String(p.account_id));
    if (!steamid64) return null;
    return {
        steamid64,
        name: p.personaname || t("voice_bans.profile_unknown"),
        avatar: p.avatarmedium ?? null,
        profileUrl: p.profileurl ?? null,
    };
}

export function chunk<T>(items: T[], size: number): T[][] {
    const out: T[][] = [];
    for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
    return out;
}

async function getJson(url: string): Promise<ApiPlayer[]> {
    const res = await fetch(url);
    if (!res.ok) throw new Error(t("voice_bans.api_error", { status: res.status }));
    return (await res.json()) as ApiPlayer[];
}

/** Players the API has never seen are simply absent from the result. */
export async function lookupProfiles(steamids64: string[], onBatch: (profiles: Profile[]) => void): Promise<void> {
    const batches = chunk(steamids64.map(steam64ToSteam32), BATCH);
    let next = 0;
    const worker = async () => {
        while (next < batches.length) {
            const ids = batches[next++];
            try {
                const players = await getJson(`${API}/steam?account_ids=${ids.join(",")}`);
                onBatch(players.map(toProfile).filter((p): p is Profile => p !== null));
            } catch {
                // A failed batch leaves those rows showing their id.
            }
        }
    };
    await Promise.all(Array.from({ length: Math.min(CONCURRENCY, batches.length) }, worker));
}

export async function searchPlayers(query: string, limit = 8): Promise<Profile[]> {
    const url = `${API}/steam-search?search_query=${encodeURIComponent(query)}&limit=${limit}`;
    return (await getJson(url)).map(toProfile).filter((p): p is Profile => p !== null);
}
