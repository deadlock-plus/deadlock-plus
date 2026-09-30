import { createPoller } from "$lib/core/poller";
import { ingestStatus as fetchIngestStatus } from "./api";

import type { IngestStatus } from "$lib/generated/types/IngestStatus";

export type { IngestStatus };

const POLL_MS = 3000;

class IngestStatusStore {
    status = $state<IngestStatus | null>(null);

    async refresh() {
        try {
            this.status = await fetchIngestStatus();
        } catch {
            // Not running inside Tauri.
        }
    }

    private poller = createPoller(() => this.refresh(), { intervalMs: POLL_MS, immediate: true });

    start() {
        return this.poller.start();
    }
}

export const ingestStatus = new IngestStatusStore();
