import { kvGet, kvSet } from "$lib/core/kv";
import type { ServerData } from "./types";

const STORE = "server-picker-cache";

export async function readCachedServerData(gameId: string): Promise<ServerData | null> {
    try {
        return (await kvGet<ServerData>(STORE, `serverData:${gameId}`)) ?? null;
    } catch {
        return null;
    }
}

export async function writeCachedServerData(gameId: string, data: ServerData): Promise<void> {
    try {
        await kvSet(STORE, `serverData:${gameId}`, data);
    } catch {
        // Best-effort cache; a failed write just means no instant-paint next launch.
    }
}
