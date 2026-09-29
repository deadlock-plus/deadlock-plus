import type { JobInfo } from "$lib/generated/types/JobInfo";
import type { JobsSnapshot } from "$lib/generated/types/JobsSnapshot";
import type { Policy } from "$lib/generated/types/Policy";

export type { JobInfo, JobsSnapshot, Policy };

/** Whether a job may start on its own. True until the first snapshot says otherwise. */
export function isJobEnabled(snapshot: JobsSnapshot | null, id: string): boolean {
    if (!snapshot) return true;
    if (!snapshot.allEnabled) return false;
    return snapshot.catalog.find((c) => c.id === id)?.enabled ?? true;
}

export const POLICY_OPTIONS: { value: Policy; label: string }[] = [
    { value: "always", label: "Keep running" },
    { value: "pauseInGame", label: "Pause" },
    { value: "slowInGame", label: "Slow down" },
];

export function activeJobs(jobs: JobInfo[]): JobInfo[] {
    return jobs.filter((j) => j.state === "queued" || j.state === "running" || j.state === "paused");
}

export function jobPercent(job: JobInfo): number {
    if (job.total <= 0) return 0;
    return Math.min(100, Math.round((job.done / job.total) * 100));
}

export function jobStatusText(job: JobInfo): string {
    if (job.state === "paused") return `${job.title} paused while Deadlock runs`;
    if (job.total <= 0) return `${job.title}...`;
    const label = job.label ? ` · ${job.label}` : "";
    return `${job.title}... ${job.done}/${job.total}${label}`;
}
