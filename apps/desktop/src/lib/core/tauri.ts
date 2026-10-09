import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Unlisten = () => void;

export function command<T = void>(name: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(name, args);
}

export function fileSrc(path: string): string {
    return convertFileSrc(path);
}

export function listen<T = void>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
    return tauriListen<T>(event, (e) => handler(e.payload));
}

export type AppWindow = ReturnType<typeof getCurrentWindow>;

export function currentWindow(): AppWindow {
    return getCurrentWindow();
}
