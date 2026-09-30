import { classifyHealth, type HealthResult } from "./health";

const URL = "https://api.deadlock-api.com/v1/info/health";
const POLL_MS = 60_000;
const TIMEOUT_MS = 10_000;

class ApiHealthStore {
    result = $state<HealthResult | null>(null);

    async refresh() {
        try {
            const res = await fetch(URL, { signal: AbortSignal.timeout(TIMEOUT_MS) });
            this.result = res.ok ? classifyHealth(await res.json()) : { level: "down", down: [] };
        } catch {
            this.result = { level: "down", down: [] };
        }
    }

    start() {
        void this.refresh();
        const timer = setInterval(() => void this.refresh(), POLL_MS);
        return () => clearInterval(timer);
    }
}

export const apiHealth = new ApiHealthStore();
