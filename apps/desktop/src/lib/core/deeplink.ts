import { openUrl } from "./opener";

const STEAM_HOSTS = ["steampowered.com", "steamcommunity.com"];

const isSteamHost = (host: string) => STEAM_HOSTS.some((h) => host === h || host.endsWith(`.${h}`));

const DEADLOCK_APP_ID = 1422450;

function steamWebUrl(url: string): URL | null {
    let parsed: URL;
    try {
        parsed = new URL(url);
    } catch {
        return null;
    }
    if (parsed.protocol !== "https:" && parsed.protocol !== "http:") return null;
    if (parsed.username || parsed.password || !isSteamHost(parsed.hostname)) return null;
    return parsed;
}

function allowedSteamLink(url: string): string | null {
    if (/^steam:\/\/store\/\d+$/.test(url) || url === `steam://run/${DEADLOCK_APP_ID}`) return url;
    const embedded = /^steam:\/\/openurl\/(.+)$/.exec(url);
    const target = embedded && steamWebUrl(embedded[1]);
    return target ? `steam://openurl/${target.href}` : null;
}

/**
 * The native-app link for a URL, or null when no app handles it. Only the forms listed here are
 * built or passed through, and each must also be allowed in the opener capability
 * (capabilities/default.json).
 */
export function deeplinkFor(url: string): string | null {
    if (url.startsWith("steam:")) return allowedSteamLink(url);
    const parsed = steamWebUrl(url);
    if (!parsed) return null;

    const app = parsed.hostname === "store.steampowered.com" && /^\/app\/(\d+)(?:\/|$)/.exec(parsed.pathname);
    return app ? `steam://store/${app[1]}` : `steam://openurl/${parsed.href}`;
}

/** Tries the app deeplink first; the browser is the fallback when the app is missing or refuses. */
export async function openWithDeeplink(url: string, open: (target: string) => Promise<void> = openUrl) {
    const deeplink = deeplinkFor(url);
    if (url.startsWith("steam:")) {
        if (!deeplink) throw new Error("unsupported steam link");
        return open(deeplink);
    }
    if (deeplink) {
        try {
            return await open(deeplink);
        } catch {
            // No handler registered (app not installed); fall through to the browser.
        }
    }
    await open(url);
}

/** Starts Deadlock through Steam; the store page opens in the browser when Steam has no handler. */
export async function launchGame(open: (target: string) => Promise<void> = openUrl) {
    try {
        return await open(`steam://run/${DEADLOCK_APP_ID}`);
    } catch {
        // Steam is not installed or refused the link.
    }
    await open(`https://store.steampowered.com/app/${DEADLOCK_APP_ID}`);
}
