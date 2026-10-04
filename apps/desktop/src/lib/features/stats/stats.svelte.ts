import { t } from "$lib/core/i18n.svelte";
import { kvGet, kvSet } from "$lib/core/kv";
import { parseStatsCache, toStatsCache, type StatsSnapshot } from "./cache";
import { parseRankInfo, parseRanks, type RankInfo, type RankTier } from "./rank";
import { getPostgameMatches, onPostgameMatch, reconcilePostgameMatches, type ProvisionalMatch } from "./postgame-api";
import { forAccount, mergeProvisional } from "./provisional";
import { createReconcileSchedule } from "./reconcile-schedule";
import { parseHistory, type Match } from "./stats";

const API = "https://api.deadlock-api.com/v1";
const CACHE_STORE = "stats-cache";
const STALE_MS = 5 * 60 * 1000;

export type StatsStatus = "idle" | "loading" | "ready" | "error";

class StatsStore {
    status = $state<StatsStatus>("idle");
    private apiMatches = $state<Match[]>([]);
    private provisional = $state<ProvisionalMatch[]>([]);
    matches = $derived<Match[]>(mergeProvisional(this.apiMatches, this.provisional));
    ranks = $state<RankTier[]>([]);
    rankInfo = $state<RankInfo | null>(null);
    error = $state<string | null>(null);
    /** Set while the data shown is a saved copy from this time (ms) because the API could not be reached. */
    cachedAt = $state<number | null>(null);
    private account: number | null = null;
    private loadedFor: number | null = null;
    private loadedAt = 0;
    private listening = false;
    private schedule = createReconcileSchedule({
        run: () => this.quietRefresh(),
        pending: () => this.provisional.length > 0,
    });

    async load(accountId: number, force = false) {
        if (this.status === "loading") return;
        this.switchAccount(accountId);
        const fresh = this.loadedFor === accountId && Date.now() - this.loadedAt < STALE_MS;
        if (fresh && !force) return;

        this.status = "loading";
        this.error = null;
        void this.listenForCaptures();
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
            if (!history.ok && matches.length === 0) throw new Error(t("stats.api_error", { status: history.status }));

            this.apply({ matches, ranks: parseRanks(ranks), rankInfo: parseRankInfo(rank) });
            this.cachedAt = null;
            void this.save(accountId);
            this.loadedFor = accountId;
            this.loadedAt = Date.now();
            this.status = "ready";
            await this.syncProvisional(matches.map((m) => m.matchId));
            this.schedule.arrived();
        } catch (e) {
            const cached = await this.readCache(accountId);
            if (cached) {
                this.apply(cached);
                this.cachedAt = cached.at;
                this.loadedFor = accountId;
                this.loadedAt = Date.now();
                this.status = "ready";
                await this.syncProvisional(null);
                this.schedule.arrived();
                return;
            }
            this.error = e instanceof Error ? e.message : String(e);
            this.status = "error";
        }
    }

    private switchAccount(accountId: number) {
        if (this.account === accountId) return;
        this.account = accountId;
        this.provisional = [];
        this.schedule.stop();
    }

    private apply({ matches, ranks, rankInfo }: StatsSnapshot) {
        this.apiMatches = matches;
        this.ranks = ranks;
        this.rankInfo = rankInfo;
    }

    private async syncProvisional(apiMatchIds: number[] | null) {
        const accountId = this.account;
        if (accountId === null) return;
        try {
            if (apiMatchIds) await reconcilePostgameMatches(accountId, apiMatchIds);
            const stored = forAccount(await getPostgameMatches(accountId), accountId);
            if (this.account === accountId) this.provisional = stored;
        } catch {
            // Capture is off or unavailable: the API history stands alone.
        }
    }

    private async quietRefresh() {
        if (this.loadedFor === null || this.status === "loading") return;
        const accountId = this.loadedFor;
        const history = await fetch(`${API}/players/${accountId}/match-history`);
        const matches = parseHistory(await history.json().catch(() => null));
        if (matches.length === 0 || this.loadedFor !== accountId) return;
        this.apiMatches = matches;
        this.cachedAt = null;
        void this.save(accountId);
        await this.syncProvisional(matches.map((m) => m.matchId));
    }

    private async listenForCaptures() {
        if (this.listening) return;
        this.listening = true;
        try {
            await onPostgameMatch((record) => {
                if (this.account === null || record.accountId !== this.account) return;
                this.provisional = [...this.provisional.filter((m) => m.matchId !== record.matchId), record];
                this.schedule.arrived();
            });
            window.addEventListener("focus", () => this.schedule.focused());
        } catch {
            this.listening = false;
        }
    }

    private async save(accountId: number) {
        try {
            const snapshot = { matches: this.apiMatches, ranks: this.ranks, rankInfo: this.rankInfo };
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
