import { invoke } from "@tauri-apps/api/core";

export function getChangelog() {
    return invoke<string>("changelog");
}
