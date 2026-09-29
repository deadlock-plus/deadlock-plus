import { kvGet, kvSet } from "$lib/kv";
import { parseStatsCache, toStatsCache, type StatsSnapshot } from "./cache";
import { parseRankInfo, parseRanks, type RankInfo, type RankTier } from "./rank";
import { parseHistory, type Match } from "./stats";

const API = "https://api.deadlock-api.com/v1";
const CACHE_STORE = "stats-cache";
const STALE_MS = 5 * 60 * 1000;

export type StatsStatus = "idle" | "loading" | "ready" | "error";

class StatsStore {
    status = $state<StatsStatus>("idle");
    matches = $state<Match[]>([]);
    ranks = $state<RankTier[]>([]);
    rankInfo = $state<RankInfo | null>(null);
    error = $state<string | null>(null);
    /** Set while the data shown is a saved copy from this time (ms) because the API could not be reached. */
    cachedAt = $state<number | null>(null);
    private loadedFor: number | null = null;
    private loadedAt = 0;

    async load(accountId: number, force = false) {
        if (this.status === "loading") return;
        const fresh = this.loadedFor === accountId && Date.now() - this.loadedAt < STALE_MS;
        if (fresh && !force) return;

        this.status = "loading";
        this.error = null;
        try {
            const [history, ranks, rank] = await Promise.all([
                fetch(`${API}/players/${accountId}/match-history`),
                fetch(`${API}/assets/ranks`)
                    .then((r) => (r.ok ? r.json() : []))
                    .catch(() => []),
                fetch(`${API}/players/${accountId}/rank`)
                    .then((r) => (r.ok ? r.json() : null))
                    .catch(() => null),
            ]);
            // A rate-limited response still carries the stored history in its body.
            const body: unknown = await history.json().catch(() => null);
            const matches = parseHistory(body);
            if (!history.ok && matches.length === 0) throw new Error(`The API answered ${history.status}.`);

            this.apply({ matches, ranks: parseRanks(ranks), rankInfo: parseRankInfo(rank) });
            this.cachedAt = null;
            void this.save(accountId);
            this.loadedFor = accountId;
            this.loadedAt = Date.now();
            this.status = "ready";
        } catch (e) {
            const cached = await this.readCache(accountId);
            if (cached) {
                this.apply(cached);
                this.cachedAt = cached.at;
                this.loadedFor = accountId;
                this.loadedAt = Date.now();
                this.status = "ready";
                return;
            }
            this.error = e instanceof Error ? e.message : String(e);
            this.status = "error";
        }
    }

    private apply({ matches, ranks, rankInfo }: StatsSnapshot) {
        this.matches = matches;
        this.ranks = ranks;
        this.rankInfo = rankInfo;
    }

    private async save(accountId: number) {
        try {
            const snapshot = { matches: this.matches, ranks: this.ranks, rankInfo: this.rankInfo };
            await kvSet(CACHE_STORE, "snapshot", toStatsCache(accountId, Date.now(), snapshot));
        } catch {
            // Best-effort: without it there is just nothing to show offline.
        }
    }

    private async readCache(accountId: number) {
        try {
            return parseStatsCache(await kvGet(CACHE_STORE, "snapshot"), accountId);
        } catch {
            return null;
        }
    }
}

export const stats = new StatsStore();
