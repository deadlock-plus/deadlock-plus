import { command } from "$lib/core/tauri";

import type { PresenceSettings } from "$lib/generated/types/PresenceSettings";
import type { PresenceStatus } from "$lib/generated/types/PresenceStatus";

export const setPresenceSettings = (settings: PresenceSettings) => command("set_presence_settings", { settings });

export const presenceStatus = () => command<PresenceStatus>("presence_status");

export const setPresenceHeroNames = (names: Record<number, string>) => command("set_presence_hero_names", { names });
