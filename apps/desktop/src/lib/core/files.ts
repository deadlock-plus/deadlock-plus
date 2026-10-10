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

export interface BinarySaveRequest {
    defaultName: string;
    extension: string;
}

/** Same contract as `saveTextFile`; the bytes travel as the raw request body so a large image is not JSON-encoded. */
export function saveBinaryFile(request: BinarySaveRequest, bytes: Uint8Array): Promise<boolean> {
    return invoke<boolean>("save_binary_file", bytes, {
        headers: { "x-default-name": request.defaultName, "x-extension": request.extension },
    });
}
