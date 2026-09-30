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

    start() {
        void this.refresh();
        const timer = setInterval(() => void this.refresh(), POLL_MS);
        return () => clearInterval(timer);
    }
}

export const ingestStatus = new IngestStatusStore();
