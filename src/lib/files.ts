import { invoke } from "@tauri-apps/api/core";

export interface SaveRequest {
    defaultName: string;
    filterName: string;
    extension: string;
}

/** The backend opens the save dialog and writes where the user picks. Resolves false when they cancel. */
export function saveTextFile(request: SaveRequest, contents: string): Promise<boolean> {
    return invoke<boolean>("save_text_file", { ...request, contents });
}
