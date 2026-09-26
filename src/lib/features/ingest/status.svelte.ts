import { invoke } from "@tauri-apps/api/core";

import type { IngestStatus } from "$lib/generated/types/IngestStatus";

export type { IngestStatus };

const POLL_MS = 3000;

class IngestStatusStore {
    status = $state<IngestStatus | null>(null);

    async refresh() {
        try {
            this.status = await invoke<IngestStatus>("ingest_status");
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
