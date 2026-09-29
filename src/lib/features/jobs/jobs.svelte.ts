import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { JobsSnapshot } from "$lib/generated/types/JobsSnapshot";
import { activeJobs, isJobEnabled, type Policy } from "./jobs";

type Listener = (snapshot: JobsSnapshot) => void;

class JobsStore {
    snapshot = $state<JobsSnapshot | null>(null);
    private listeners = new Set<Listener>();

    active = $derived(activeJobs(this.snapshot?.jobs ?? []));
    gameRunning = $derived(this.snapshot?.gameRunning ?? null);
    catalog = $derived(this.snapshot?.catalog ?? []);
    pauseInGame = $derived(this.snapshot?.pauseInGame ?? true);
    allEnabled = $derived(this.snapshot?.allEnabled ?? true);

    isActive(id: string): boolean {
        return this.active.some((j) => j.id === id);
    }

    isEnabled(id: string): boolean {
        return isJobEnabled(this.snapshot, id);
    }

    /** Runs on every snapshot, including the ones a refresh fetches. Returns an unsubscribe. */
    onChange(listener: Listener) {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    private apply(snapshot: JobsSnapshot) {
        this.snapshot = snapshot;
        this.listeners.forEach((l) => l(snapshot));
    }

    async refresh() {
        try {
            this.apply(await invoke<JobsSnapshot>("jobs_snapshot"));
        } catch {
            // Not running inside Tauri.
        }
    }

    async setPolicy(id: string, policy: Policy) {
        await invoke("set_job_policy", { id, policy });
        await this.refresh();
    }

    async setPauseInGame(enabled: boolean) {
        await invoke("set_pause_in_game", { enabled });
        await this.refresh();
    }

    async setEnabled(id: string, enabled: boolean) {
        await invoke("set_job_enabled", { id, enabled });
        await this.refresh();
    }

    async setAllEnabled(enabled: boolean) {
        await invoke("set_all_jobs_enabled", { enabled });
        await this.refresh();
    }

    async forceRun(id: string) {
        await invoke("force_run_job", { id });
        await this.refresh();
    }

    start() {
        void this.refresh();
        let stopped = false;
        let unlisten: (() => void) | undefined;
        listen<JobsSnapshot>("jobs://changed", (e) => this.apply(e.payload)).then(
            (fn) => (stopped ? fn() : (unlisten = fn)),
            () => {},
        );
        return () => {
            stopped = true;
            unlisten?.();
        };
    }
}

export const jobs = new JobsStore();
