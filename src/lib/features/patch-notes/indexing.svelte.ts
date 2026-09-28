import { invoke } from "@tauri-apps/api/core";

import type { IndexingProgress } from "$lib/generated/types/IndexingProgress";

export type { IndexingProgress };

const POLL_MS = 2000;

class PatchNotesIndexingStore {
    progress = $state<IndexingProgress | null>(null);

    async refresh() {
        try {
            this.progress = await invoke<IndexingProgress>("patch_notes_indexing_progress");
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

export const patchNotesIndexing = new PatchNotesIndexingStore();
