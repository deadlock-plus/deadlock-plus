import { loadHeroes, type Hero } from "$lib/features/heroes/heroes";
import type { RankTier } from "$lib/features/stats/rank";
import { getLiveMatch, getLiveState, onLiveMatch, onLiveSnapshot, type LiveMatch, type LiveState } from "./api";
import { BOARD_LINGER_MS, boardVisible, postMatchStart } from "./live";
import { loadRankTiers } from "./ranks";

class LiveStore {
    state = $state<LiveState | null>(null);
    match = $state<LiveMatch | null>(null);
    heroes = $state<Record<number, Hero>>({});
    tiers = $state<RankTier[]>([]);
    private postMatchAt = $state<number | null>(null);
    private now = $state(Date.now());
    private timer: ReturnType<typeof setTimeout> | null = null;

    readonly boardShown = $derived(
        this.state !== null &&
            boardVisible(this.state.phase, this.postMatchAt === null ? null : this.now - this.postMatchAt),
    );

    private apply(next: LiveState) {
        this.now = Date.now();
        this.postMatchAt = postMatchStart(next.phase, this.postMatchAt, this.now);
        this.state = next;
        this.schedule();
    }

    private schedule() {
        if (this.timer) clearTimeout(this.timer);
        this.timer = null;
        if (this.postMatchAt === null) return;
        const left = this.postMatchAt + BOARD_LINGER_MS - Date.now();
        if (left <= 0) {
            this.now = Date.now();
            return;
        }
        this.timer = setTimeout(() => {
            this.timer = null;
            this.now = Date.now();
        }, left);
    }

    /** Hydrates once, then follows pushed events. Returns the cleanup. */
    start() {
        let stopped = false;
        const unlisteners: Array<() => void> = [];
        const keep = (fn: () => void) => {
            if (stopped) fn();
            else unlisteners.push(fn);
        };

        onLiveSnapshot((next) => {
            if (!stopped) this.apply(next);
        })
            .then(keep)
            .catch(() => {});
        onLiveMatch((next) => {
            if (!stopped) this.match = next;
        })
            .then(keep)
            .catch(() => {});

        getLiveState()
            .then((initial) => {
                // A pushed snapshot may already be newer than this reply.
                if (!stopped && this.state === null) this.apply(initial);
            })
            .catch(() => {});
        getLiveMatch()
            .then((initial) => {
                if (!stopped && this.match === null) this.match = initial;
            })
            .catch(() => {});

        void loadHeroes().then((h) => {
            if (!stopped) this.heroes = h;
        });
        void loadRankTiers().then((r) => {
            if (!stopped) this.tiers = r;
        });

        return () => {
            stopped = true;
            if (this.timer) clearTimeout(this.timer);
            this.timer = null;
            for (const fn of unlisteners) fn();
        };
    }
}

export const live = new LiveStore();
