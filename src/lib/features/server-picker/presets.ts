import { kvGet, kvSet } from "$lib/kv";

import { DEFAULT_PRESETS } from "./default-presets";

export type PresetMode = "allow" | "block";

export interface Preset {
    id: string;
    name: string;
    /** "allow": only the listed regions stay open, the rest are blocked. "block": only the listed regions are blocked. */
    mode: PresetMode;
    regionIds: string[];
}

export function resolveBlockedIds(preset: Preset, allRegionIds: string[]): string[] {
    const listed = new Set(preset.regionIds);
    return allRegionIds.filter((id) => (preset.mode === "allow" ? !listed.has(id) : listed.has(id)));
}

export function diffBlocks(target: string[], current: Set<string>): { toBlock: string[]; toUnblock: string[] } {
    const wanted = new Set(target);
    return {
        toBlock: target.filter((id) => !current.has(id)),
        toUnblock: [...current].filter((id) => !wanted.has(id)),
    };
}

/** Defaults go in once, on a first install. Any stored list (even an empty one) or a set flag means never again. */
export function seedPresets(stored: Preset[] | null, seeded: boolean, defaults: Preset[]): Preset[] {
    if (seeded || stored !== null) return stored ?? [];
    return defaults.map((p) => ({ ...p, regionIds: [...p.regionIds] }));
}

const STORE = "presets";
const KEY = "presets";
const SEEDED_KEY = "defaultsSeeded";

export async function readPresets(): Promise<Preset[]> {
    try {
        const stored = await kvGet<Preset[]>(STORE, KEY);
        const seeded = (await kvGet<boolean>(STORE, SEEDED_KEY)) === true;
        const presets = seedPresets(stored, seeded, DEFAULT_PRESETS);
        if (!seeded) {
            if (stored === null) await kvSet(STORE, KEY, presets);
            await kvSet(STORE, SEEDED_KEY, true);
        }
        return presets;
    } catch {
        return [];
    }
}

export async function writePresets(presets: Preset[]): Promise<void> {
    await kvSet(STORE, KEY, presets);
}
