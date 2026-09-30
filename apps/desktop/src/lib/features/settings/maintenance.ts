import type { MaintenanceSchedule } from "$lib/generated/types/MaintenanceSchedule";

export type { MaintenanceSchedule };

export const DEFAULT_SCHEDULE: MaintenanceSchedule = {
    enabled: false,
    weekday: 2,
    minuteOfDay: 0,
    leadMinutes: 30,
};

export const WEEKDAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
export const LEAD_OPTIONS = [15, 30, 60, 120];

export function parseTime(value: string): number | null {
    const m = /^(\d{1,2}):(\d{2})$/.exec(value.trim());
    if (!m) return null;
    const h = Number(m[1]);
    const min = Number(m[2]);
    return h < 24 && min < 60 ? h * 60 + min : null;
}

export function formatTime(minuteOfDay: number): string {
    const h = Math.floor(minuteOfDay / 60);
    const m = minuteOfDay % 60;
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

export function countdown(seconds: number): string {
    const total = Math.max(0, Math.floor(seconds / 60));
    const d = Math.floor(total / 1440);
    const h = Math.floor((total % 1440) / 60);
    const m = total % 60;
    if (d > 0) return `${d} d ${h} h`;
    if (h > 0) return `${h} h ${m} min`;
    if (m > 0) return `${m} min`;
    return "less than a minute";
}

export { nextMaintenance } from "./api";
