import { parseRanks, type RankTier } from "$lib/features/stats/rank";

const RANKS_URL = "https://api.deadlock-api.com/v1/assets/ranks";

export async function loadRankTiers(): Promise<RankTier[]> {
    try {
        const res = await fetch(RANKS_URL);
        return res.ok ? parseRanks(await res.json()) : [];
    } catch {
        return [];
    }
}
