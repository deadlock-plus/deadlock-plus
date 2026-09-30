import { kvGet, kvSet } from "$lib/core/kv";
import { parseSort, type SortState } from "./sort";

const STORE = "app-settings";
const KEY = "serverPickerSort";

export async function readSort(): Promise<SortState> {
    try {
        return parseSort(await kvGet<unknown>(STORE, KEY));
    } catch {
        return parseSort(null);
    }
}

export async function writeSort(sort: SortState): Promise<void> {
    try {
        await kvSet(STORE, KEY, sort);
    } catch {
        // The choice just won't persist across restarts.
    }
}
