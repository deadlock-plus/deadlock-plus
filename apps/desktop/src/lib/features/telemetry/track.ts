// Must stay in step with `FEATURES` in dp-telemetry: the backend drops any name it does not list.
const TRACKED = new Set([
    "server-picker",
    "connection",
    "stats",
    "rank",
    "sessions",
    "alerts",
    "performance",
    "voice-bans",
    "demos",
    "storage",
]);

export function featureForPath(pathname: string): string | null {
    const segment = pathname.split("/").find((part) => part !== "") ?? "";
    if (segment === "") return "home";
    if (segment === "settings") return "settings";
    return TRACKED.has(segment) ? segment : null;
}

export function trackRoute(pathname: string, send: (feature: string) => Promise<unknown>) {
    const feature = featureForPath(pathname);
    if (feature) send(feature).catch(() => {});
}
