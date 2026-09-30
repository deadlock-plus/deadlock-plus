import { command } from "$lib/core/tauri";
import type { ClearReport, EntryId, EntryInfo, EntryStats } from "./storage";

export function storageEntries() {
    return command<EntryInfo[]>("storage_entries");
}

export function storageEntryStats(id: EntryId) {
    return command<EntryStats>("storage_entry_stats", { id });
}

export function storageReveal(id: EntryId) {
    return command<void>("storage_reveal", { id });
}

export function storageClear(id: EntryId) {
    return command<ClearReport>("storage_clear", { id });
}
