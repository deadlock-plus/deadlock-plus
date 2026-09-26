import { kvGet, kvSet } from "$lib/kv";

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

const STORE = "presets";
const KEY = "presets";

export async function readPresets(): Promise<Preset[]> {
    try {
        return (await kvGet<Preset[]>(STORE, KEY)) ?? [];
    } catch {
        return [];
    }
}

export async function writePresets(presets: Preset[]): Promise<void> {
    await kvSet(STORE, KEY, presets);
}
