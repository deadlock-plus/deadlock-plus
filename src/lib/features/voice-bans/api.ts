import { invoke } from "@tauri-apps/api/core";

import type { VoiceBanFile } from "$lib/generated/types/VoiceBanFile";

export type { VoiceBanFile };

export function readVoiceBan() {
    return invoke<VoiceBanFile>("read_voice_ban");
}

export function writeVoiceBan(text: string) {
    return invoke<string>("write_voice_ban", { text });
}
