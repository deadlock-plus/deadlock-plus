import type { Eta } from "./climb";

export const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);

export const pct = (v: number | null) => (v === null ? "-" : `${Math.round(v * 100)}%`);

export const shortDay = (s: number) =>
    new Date(s * 1000).toLocaleDateString(undefined, { day: "numeric", month: "short" });

export function etaText(e: Eta): string {
    if (e.daysLow === null) return "Not climbing at your recent pace";
    if (e.daysHigh === null) return `${e.daysLow} days or more`;
    return e.daysLow === e.daysHigh
        ? `About ${e.daysLow} ${e.daysLow === 1 ? "day" : "days"}`
        : `${e.daysLow} to ${e.daysHigh} days`;
}
