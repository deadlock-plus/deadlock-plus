import { t } from "$lib/core/i18n.svelte";
import { command } from "$lib/core/tauri";
import { parseApiDetail } from "./api-detail";
import type { MatchDetail } from "./detail";

export const API_BASE = "https://api.deadlock-api.com/v1";

export const readDetailCache = (matchId: number) => command<string | null>("match_detail_cache_read", { matchId });
export const writeDetailCache = (matchId: number, json: string) =>
    command("match_detail_cache_write", { matchId, json });

export async function fetchMetadata(matchId: number, fetchFn: typeof fetch = fetch): Promise<string> {
    const res = await fetchFn(`${API_BASE}/matches/${matchId}/metadata`);
    if (!res.ok) throw new Error(t("match_history.api_error", { status: res.status }));
    return res.text();
}

export interface DetailSource {
    read(matchId: number): Promise<string | null>;
    write(matchId: number, json: string): Promise<void>;
    fetchText(matchId: number): Promise<string>;
}

export const liveDetailSource: DetailSource = {
    read: readDetailCache,
    write: writeDetailCache,
    fetchText: (matchId) => fetchMetadata(matchId),
};

function parseBody(text: string | null, matchId: number): MatchDetail | null {
    if (text === null) return null;
    try {
        const detail = parseApiDetail(JSON.parse(text));
        return detail?.matchId === matchId ? detail : null;
    } catch {
        return null;
    }
}

/**
 * Cache first; a missing, unreadable or unparseable cache entry falls through to one fetch.
 * Rejects when the fetched body is not a usable match.
 */
export async function resolveDetail(matchId: number, source: DetailSource = liveDetailSource): Promise<MatchDetail> {
    const cached = await source.read(matchId).catch(() => null);
    const hit = parseBody(cached, matchId);
    if (hit) return hit;

    const text = await source.fetchText(matchId);
    const detail = parseBody(text, matchId);
    if (!detail) throw new Error(t("match_history.detail_invalid"));
    // Best-effort: a failed write only costs a refetch next time.
    await source.write(matchId, text).catch(() => {});
    return detail;
}
