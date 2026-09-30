import { openUrl } from "./opener";

const STEAM_HOSTS = ["steampowered.com", "steamcommunity.com"];

const isSteamHost = (host: string) => STEAM_HOSTS.some((h) => host === h || host.endsWith(`.${h}`));

/**
 * The native-app link for a URL, or null when no app handles it. Each scheme built here must
 * also be allowed in the opener capability (capabilities/default.json).
 */
export function deeplinkFor(url: string): string | null {
    let parsed: URL;
    try {
        parsed = new URL(url);
    } catch {
        return null;
    }
    if (parsed.protocol === "steam:") return url;
    if (parsed.protocol !== "https:" && parsed.protocol !== "http:") return null;
    if (!isSteamHost(parsed.hostname)) return null;

    const app = parsed.hostname === "store.steampowered.com" && /^\/app\/(\d+)(?:\/|$)/.exec(parsed.pathname);
    return app ? `steam://store/${app[1]}` : `steam://openurl/${url}`;
}

/** Tries the app deeplink first; the browser is the fallback when the app is missing or refuses. */
export async function openWithDeeplink(url: string, open: (target: string) => Promise<void> = openUrl) {
    const deeplink = deeplinkFor(url);
    if (deeplink === url) return open(url);
    if (deeplink) {
        try {
            return await open(deeplink);
        } catch {
            // No handler registered (app not installed); fall through to the browser.
        }
    }
    await open(url);
}
