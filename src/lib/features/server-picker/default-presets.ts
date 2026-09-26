import type { Preset } from "./presets";

const files = import.meta.glob<Preset>("./default-presets/*.json", { eager: true, import: "default" });

export const DEFAULT_PRESETS: Preset[] = Object.values(files).sort((a, b) => a.name.localeCompare(b.name));
