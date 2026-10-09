import { command, fileSrc } from "$lib/core/tauri";
import type { ClassArt } from "$lib/generated/types/ClassArt";
import type { ItemImage } from "$lib/generated/types/ItemImage";

const BATCH = 200;

async function load(name: string, names: string[], extra: Record<string, unknown>): Promise<Record<string, string>> {
    const out: Record<string, string> = {};
    for (let i = 0; i < names.length; i += BATCH) {
        try {
            const found = await command<ClassArt[]>(name, { names: names.slice(i, i + BATCH), ...extra });
            for (const a of found) out[a.className] = fileSrc(a.path);
        } catch {
            // A failed batch only loses its own images; callers fall back to remote art.
        }
    }
    return out;
}

/** Asset urls of item images decoded from the installed game, keyed by class name; absent names have none. */
export function loadItemArt(names: string[], kind: ItemImage): Promise<Record<string, string>> {
    return load("game_item_art", names, { kind });
}

/** Asset urls of ability icons from the installed game. Vector-only icons have no local file and are absent. */
export function loadAbilityArt(names: string[]): Promise<Record<string, string>> {
    return load("game_ability_art", names, {});
}
