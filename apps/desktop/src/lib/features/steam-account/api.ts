import { command } from "$lib/core/tauri";
import type { SteamAccount } from "$lib/generated/types/SteamAccount";

export function currentSteamAccount() {
    return command<SteamAccount | null>("current_steam_account");
}
