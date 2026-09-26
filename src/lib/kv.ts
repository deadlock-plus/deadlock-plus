import { invoke } from "@tauri-apps/api/core";

export type KvStore = "app-settings" | "connection-settings" | "server-picker-cache" | "presets" | "stats-cache";

export async function kvGet<T>(store: KvStore, key: string): Promise<T | null> {
    return (await invoke<T | null>("kv_get", { store, key })) ?? null;
}

export function kvSet(store: KvStore, key: string, value: unknown): Promise<void> {
    return invoke("kv_set", { store, key, value });
}

export function kvDelete(store: KvStore, key: string): Promise<void> {
    return invoke("kv_delete", { store, key });
}
