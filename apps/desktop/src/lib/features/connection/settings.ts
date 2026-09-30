import { kvDelete, kvGet, kvSet } from "$lib/core/kv";

const STORE = "connection-settings";
const OFFSET_KEY = "exitlagOffsetMs";

export async function readExitLagOffset(): Promise<number | null> {
    try {
        return (await kvGet<number>(STORE, OFFSET_KEY)) ?? null;
    } catch {
        return null;
    }
}

export async function writeExitLagOffset(offset: number | null): Promise<void> {
    try {
        if (offset === null) await kvDelete(STORE, OFFSET_KEY);
        else await kvSet(STORE, OFFSET_KEY, offset);
    } catch {
        // Calibration just won't persist across restarts.
    }
}
