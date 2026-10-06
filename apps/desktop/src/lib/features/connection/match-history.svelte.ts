import { untrack } from "svelte";

import { live } from "$lib/features/live/live.svelte";

import { networkHistoryRange, networkSnapshot } from "./api";
import {
    addMatch,
    finishedMatch,
    readMatches,
    stepMatch,
    writeMatches,
    type MatchRecord,
    type OpenMatch,
} from "./match-history";

class MatchHistoryStore {
    matches = $state<MatchRecord[]>([]);
    private open: OpenMatch | null = null;
    private loaded: Promise<void> = Promise.resolve();

    private async captureServer(target: OpenMatch) {
        try {
            const relay = (await networkSnapshot()).relay;
            target.server = relay?.description ?? relay?.popCode ?? null;
        } catch {
            // The match is still recorded, just without a server name.
        }
    }

    private async finish(closed: OpenMatch, endedAt: number) {
        try {
            const summary = await networkHistoryRange(closed.startedAt, endedAt);
            const record = finishedMatch(closed, endedAt, summary);
            if (record === null) return;
            await this.loaded;
            this.matches = addMatch(this.matches, record);
            await writeMatches(this.matches);
        } catch {
            // Match history is a convenience; a failed read or save just skips this match.
        }
    }

    private onPhase(phase: Parameters<typeof stepMatch>[1]) {
        const now = Date.now();
        const wasOpen = this.open;
        const { open, closed } = stepMatch(this.open, phase, now);
        this.open = open;
        if (open && !wasOpen) void this.captureServer(open);
        if (closed) void this.finish(closed, now);
    }

    /** Follows `live.state`, so the live store must be running. Returns the cleanup. */
    start() {
        this.loaded = readMatches().then((list) => {
            this.matches = list;
        });
        return $effect.root(() => {
            $effect(() => {
                const phase = live.state?.phase;
                untrack(() => this.onPhase(phase));
            });
        });
    }
}

export const matchHistory = new MatchHistoryStore();
