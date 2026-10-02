export function returnTarget(from: { url?: URL | null } | null | undefined): string | null {
    const url = from?.url;
    if (!url || url.pathname.startsWith("/settings")) return null;
    return url.pathname + url.search;
}
