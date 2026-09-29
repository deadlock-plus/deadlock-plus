import type { FrameStats } from "./api";
import { addRun, makeRun, readRuns, removeRun, writeRuns, type SavedRun } from "./runs";

class SavedRunsStore {
    runs = $state<SavedRun[]>([]);
    error = $state<string | null>(null);
    loaded = false;

    async load() {
        if (this.loaded) return;
        this.loaded = true;
        this.runs = await readRuns();
    }

    async save(label: string, addons: string[], stats: FrameStats): Promise<boolean> {
        return this.commit(addRun(this.runs, makeRun(label, addons, stats, Date.now(), crypto.randomUUID())));
    }

    async remove(id: string): Promise<boolean> {
        return this.commit(removeRun(this.runs, id));
    }

    private async commit(next: SavedRun[]): Promise<boolean> {
        this.error = null;
        try {
            await writeRuns(next);
            this.runs = next;
            return true;
        } catch (e) {
            this.error = String(e);
            return false;
        }
    }
}

export const savedRuns = new SavedRunsStore();
