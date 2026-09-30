export type HealthLevel = "ok" | "degraded" | "down";

export interface HealthResult {
    level: HealthLevel;
    down: string[];
}

export function classifyHealth(body: unknown): HealthResult {
    const services = (body as { services?: unknown } | null)?.services;
    if (!services || typeof services !== "object") return { level: "down", down: [] };
    const down = Object.entries(services)
        .filter(([, up]) => up !== true)
        .map(([name]) => name);
    return { level: down.length ? "degraded" : "ok", down };
}
