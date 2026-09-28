import { invoke } from "@tauri-apps/api/core";

import { formatBytes } from "$lib/features/demos/demos";

import type { EntryId } from "$lib/generated/types/EntryId";
import type { EntryInfo } from "$lib/generated/types/EntryInfo";
import type { ClearReport } from "$lib/generated/types/ClearReport";

export type { EntryId, EntryInfo, ClearReport };

export const ENTRY_IDS: EntryId[] = [
    "replays",
    "addons",
    "addons-backups",
    "config-backups",
    "gameinfo-backups",
    "hero-presence-cache",
    "shader-cache",
    "console-log",
    "voice-ban-backups",
    "app-data",
    "logs",
];

interface EntryMeta {
    label: string;
    description: string;
    link?: string;
}

export const ENTRY_META: Record<EntryId, EntryMeta> = {
    replays: {
        label: "Replays",
        description: "Match replays saved by Deadlock. Manage them on the Replays page.",
        link: "/demos",
    },
    addons: {
        label: "Mods and addons",
        description: "Files installed by mods or a mod manager, without the replays. Not touched here.",
    },
    "addons-backups": {
        label: "Addon backups",
        description: "Backup copies of addons made by a mod manager. Not touched here.",
    },
    "config-backups": {
        label: "Config backups",
        description: "Config folders backed up by the game or a mod manager. Not touched here.",
    },
    "gameinfo-backups": {
        label: "Game setup backups",
        description: "Copies of gameinfo.gi made by mod tools. Not touched here.",
    },
    "hero-presence-cache": {
        label: "Hero presence cache",
        description: "A small cache file kept by the game.",
    },
    "shader-cache": {
        label: "Shader cache",
        description: "Compiled shaders. The game rebuilds them, so the first match after clearing may stutter.",
    },
    "console-log": {
        label: "Console log",
        description: "Only exists when the game starts with -condebug. Rewritten each launch.",
    },
    "voice-ban-backups": {
        label: "Mute list backups",
        description: "Backups Deadlock+ makes each time you change your mute list.",
    },
    "app-data": {
        label: "Deadlock+ data",
        description: "Your pins, cleanup rules and connection history.",
    },
    logs: {
        label: "Deadlock+ logs",
        description: "Deadlock+'s own log files. Clearing here removes old archived days; today's log stays put.",
    },
};

const CLEAR_NAME: Partial<Record<EntryId, string>> = {
    "shader-cache": "the shader cache",
    "console-log": "the console log",
    "voice-ban-backups": "your mute list backups",
    logs: "old log archives",
};

export function clearCopy(id: EntryId, bytes: number): { title: string; body: string } {
    const name = CLEAR_NAME[id] ?? ENTRY_META[id].label.toLowerCase();
    return {
        title: `Clear ${name}?`,
        body: `Frees ${formatBytes(bytes)}. ${ENTRY_META[id].description} This can't be undone.`,
    };
}

export function knownTotal(sizes: Partial<Record<EntryId, number>>): number {
    return Object.values(sizes).reduce((sum, n) => sum + (n ?? 0), 0);
}

export function storageEntries() {
    return invoke<EntryInfo[]>("storage_entries");
}

export function storageEntrySize(id: EntryId) {
    return invoke<number>("storage_entry_size", { id });
}

export function storageReveal(id: EntryId) {
    return invoke<void>("storage_reveal", { id });
}

export function storageClear(id: EntryId) {
    return invoke<ClearReport>("storage_clear", { id });
}
