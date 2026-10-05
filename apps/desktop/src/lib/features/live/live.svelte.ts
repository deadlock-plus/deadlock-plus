import { getLiveState, onLiveSnapshot, type LiveState } from "./api";

class LiveStore {
    state = $state<LiveState | null>(null);

    /** Hydrates once, then follows pushed snapshots. Returns the cleanup. */
    start() {
        let stopped = false;
        let unlisten: (() => void) | null = null;

        onLiveSnapshot((next) => {
            if (!stopped) this.state = next;
        })
            .then((fn) => {
                if (stopped) fn();
                else unlisten = fn;
            })
            .catch(() => {});

        getLiveState()
            .then((initial) => {
                // A pushed snapshot may already be newer than this reply.
                if (!stopped && this.state === null) this.state = initial;
            })
            .catch(() => {});

        return () => {
            stopped = true;
            unlisten?.();
        };
    }
}

export const live = new LiveStore();
