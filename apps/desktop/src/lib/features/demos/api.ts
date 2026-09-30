import { command } from "$lib/core/tauri";

import type { CleanupMatch } from "$lib/generated/types/CleanupMatch";
import type { CleanupRule } from "$lib/generated/types/CleanupRule";
import type { DeleteMode } from "$lib/generated/types/DeleteMode";
import type { DeletePreview } from "$lib/generated/types/DeletePreview";
import type { DeleteReport } from "$lib/generated/types/DeleteReport";
import type { Demo } from "$lib/generated/types/Demo";
import type { DemoListing } from "$lib/generated/types/DemoListing";
import type { MetaResult } from "$lib/generated/types/MetaResult";

export function fetchDemoMetadata(matchId: number) {
    return command<MetaResult>("demo_metadata", { matchId });
}

export function localAccountIds() {
    return command<number[]>("local_steam_account_ids");
}

export function listDemos() {
    return command<DemoListing>("list_demos");
}

export function openReplaysDir() {
    return command<void>("open_replays_dir");
}

export function revealDemo(demo: Demo) {
    return command<void>("reveal_demo", { matchId: String(demo.matchId), partial: demo.status === "partial" });
}

export function previewDelete(fileNames: string[]) {
    return command<DeletePreview>("delete_preview", { fileNames });
}

export function deleteDemos(fileNames: string[], mode: DeleteMode) {
    return command<DeleteReport>("delete_demos", { fileNames, mode });
}

export function listPinned() {
    return command<number[]>("list_pinned");
}

export function setPinned(matchId: number, pinned: boolean) {
    return command<number[]>("set_pinned", { matchId, pinned });
}

export function listCleanupRules() {
    return command<CleanupRule[]>("list_cleanup_rules");
}

export function saveCleanupRules(rules: CleanupRule[]) {
    return command<void>("save_cleanup_rules", { rules });
}

export function cleanupMatches() {
    return command<CleanupMatch[]>("cleanup_matches");
}
