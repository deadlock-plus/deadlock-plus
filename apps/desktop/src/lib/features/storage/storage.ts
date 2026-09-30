import { formatBytes } from "$lib/features/demos/demos";
import type { BadgeVariant } from "$lib/ui/badge.svelte";

import type { EntryId } from "$lib/generated/types/EntryId";
import type { EntryInfo } from "$lib/generated/types/EntryInfo";
import type { EntryStats } from "$lib/generated/types/EntryStats";
import type { ClearReport } from "$lib/generated/types/ClearReport";

export type { EntryId, EntryInfo, EntryStats, ClearReport };

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
    "frame-runs",
    "settings",
    "server-presets",
    "replay-rules",
    "connection-history",
    "notifications",
    "patch-notes-index",
    "stats-cache",
    "server-list-cache",
    "replay-info-cache",
    "other-app-files",
    "logs",
];

export type OwnerId = "deadlock" | "deadlock-plus" | "mods";
export type Kind = "regenerates" | "history" | "backup" | "yours" | "managed";

export const OWNERS: { id: OwnerId; label: string; blurb: string }[] = [
    { id: "deadlock", label: "Deadlock", blurb: "Files the game keeps on this PC." },
    { id: "deadlock-plus", label: "Deadlock+", blurb: "What this app keeps." },
    { id: "mods", label: "Mods and backups", blurb: "Handled by your mod tools. Not touched here." },
];

export const KIND_META: Record<Kind, { label: string; hint: string; variant: BadgeVariant }> = {
    regenerates: { label: "Regenerates", hint: "Rebuilt automatically when needed.", variant: "success" },
    history: { label: "History", hint: "A record of the past. Can't be rebuilt once deleted.", variant: "warning" },
    backup: { label: "Backup", hint: "A safety copy. Only needed if something goes wrong.", variant: "secondary" },
    yours: { label: "Yours", hint: "Your own settings and data.", variant: "default" },
    managed: { label: "Managed by mods", hint: "Belongs to a mod or a mod manager.", variant: "outline" },
};

interface EntryMeta {
    label: string;
    description: string;
    consequence: string;
    owner: OwnerId;
    kind: Kind;
    noun?: { one: string; other: string };
    link?: string;
}

export const ENTRY_META: Record<EntryId, EntryMeta> = {
    replays: {
        label: "Replays",
        description: "Match replays saved by Deadlock.",
        consequence: "Deleted replays are gone for good. Pick which ones to remove on the Replays page.",
        owner: "deadlock",
        kind: "history",
        noun: { one: "replay", other: "replays" },
        link: "/demos",
    },
    "hero-presence-cache": {
        label: "Hero presence cache",
        description: "A small cache file kept by the game.",
        consequence: "Rebuilt on the next launch. Too small to be worth clearing.",
        owner: "deadlock",
        kind: "regenerates",
    },
    "shader-cache": {
        label: "Shader cache",
        description: "Compiled shaders, so the game doesn't rebuild them every match.",
        consequence: "The game rebuilds it. The first match after clearing may stutter.",
        owner: "deadlock",
        kind: "regenerates",
    },
    "console-log": {
        label: "Console log",
        description: "The game's console output. Only exists when it starts with -condebug.",
        consequence: "Rewritten on every launch. Nothing you need is lost.",
        owner: "deadlock",
        kind: "regenerates",
    },
    "voice-ban-backups": {
        label: "Mute list backups",
        description: "Copies Deadlock+ makes each time you change your mute list.",
        consequence: "Clearing them removes your way back to an earlier mute list.",
        owner: "deadlock-plus",
        kind: "backup",
        noun: { one: "backup", other: "backups" },
    },
    "frame-runs": {
        label: "Frametime runs",
        description: "Your saved frametime captures.",
        consequence: "Can't be recaptured. Delete single runs on the Performance page.",
        owner: "deadlock-plus",
        kind: "history",
        link: "/performance",
    },
    settings: {
        label: "Settings",
        description: "Your Deadlock+ settings and window position.",
        consequence: "Not recoverable. Removing it resets your preferences.",
        owner: "deadlock-plus",
        kind: "yours",
    },
    "server-presets": {
        label: "Server presets",
        description: "Your saved Server Picker presets.",
        consequence: "Not recoverable. You would have to build them again.",
        owner: "deadlock-plus",
        kind: "yours",
    },
    "replay-rules": {
        label: "Replay pins and cleanup rules",
        description: "Replays you pinned and your automatic cleanup rules.",
        consequence: "Without it, pinned replays lose their protection and cleanup rules reset.",
        owner: "deadlock-plus",
        kind: "yours",
    },
    "connection-history": {
        label: "Connection history",
        description: "Ping and route samples recorded on the Connection page.",
        consequence: "Past samples can't be recorded again. New ones start piling up from scratch.",
        owner: "deadlock-plus",
        kind: "history",
    },
    notifications: {
        label: "Alerts and notifications",
        description: "Alerts and notifications shown in the app.",
        consequence: "Past entries are gone. Nothing else depends on them.",
        owner: "deadlock-plus",
        kind: "history",
    },
    "patch-notes-index": {
        label: "Patch notes index",
        description: "Patch notes and the search index built from them.",
        consequence: "Fetched and indexed again on its own, which takes a while and uses CPU.",
        owner: "deadlock-plus",
        kind: "regenerates",
    },
    "stats-cache": {
        label: "Stats cache",
        description: "Stats and match data fetched from the web.",
        consequence: "Fetched again the next time you open Stats.",
        owner: "deadlock-plus",
        kind: "regenerates",
    },
    "server-list-cache": {
        label: "Server list cache",
        description: "The last server list Server Picker loaded.",
        consequence: "Fetched again the next time you open Server Picker.",
        owner: "deadlock-plus",
        kind: "regenerates",
    },
    "replay-info-cache": {
        label: "Replay info cache",
        description: "Match details looked up for your replays.",
        consequence: "Looked up again when you open a replay.",
        owner: "deadlock-plus",
        kind: "regenerates",
    },
    "other-app-files": {
        label: "Other app files",
        description: "Small state files that don't belong to anything above.",
        consequence: "Reminders and leftovers. Not worth touching.",
        owner: "deadlock-plus",
        kind: "yours",
    },
    logs: {
        label: "App logs",
        description: "This app's log files, one archive per day.",
        consequence: "Only useful for bug reports. Clearing removes old days; today's log stays.",
        owner: "deadlock-plus",
        kind: "history",
        noun: { one: "log file", other: "log files" },
    },
    addons: {
        label: "Installed mods",
        description: "Files installed by mods or a mod manager, without the replays.",
        consequence: "Removing files here can break installed mods. Use your mod manager instead.",
        owner: "mods",
        kind: "managed",
    },
    "addons-backups": {
        label: "Mod backups",
        description: "Backup copies of mods made by a mod manager.",
        consequence: "Without them, you can't roll mods back to an earlier state.",
        owner: "mods",
        kind: "backup",
        noun: { one: "backup", other: "backups" },
    },
    "config-backups": {
        label: "Config backups",
        description: "Config folders backed up by the game or a mod manager.",
        consequence: "Snapshots of your settings from before a change. Keep the latest one.",
        owner: "mods",
        kind: "backup",
        noun: { one: "backup", other: "backups" },
    },
    "gameinfo-backups": {
        label: "gameinfo.gi backups",
        description: "Copies of gameinfo.gi made by mod tools.",
        consequence: "Restore points for the file that loads mods. Keep at least one.",
        owner: "mods",
        kind: "backup",
        noun: { one: "backup", other: "backups" },
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
        body: `Frees ${formatBytes(bytes)}. ${ENTRY_META[id].consequence} This can't be undone.`,
    };
}

export function clearAllCopy(ids: EntryId[], stats: StatsById): { title: string; body: string } {
    const bytes = ids.reduce((sum, id) => sum + (stats[id]?.bytes ?? 0), 0);
    const names = ids.map((id) => ENTRY_META[id].label).join(", ");
    return {
        title: "Clear all regenerable files?",
        body: `Frees ${formatBytes(bytes)} from: ${names}. The game rebuilds these on its own. This can't be undone.`,
    };
}

export type StatsById = Partial<Record<EntryId, EntryStats>>;

export function knownTotal(stats: StatsById): number {
    return Object.values(stats).reduce((sum, s) => sum + (s?.bytes ?? 0), 0);
}

export interface Group {
    owner: (typeof OWNERS)[number];
    entries: EntryInfo[];
    bytes: number;
}

function bySizeThenOrder(a: { bytes?: number; index: number }, b: { bytes?: number; index: number }) {
    if (a.bytes === undefined || b.bytes === undefined) {
        if (a.bytes === b.bytes) return a.index - b.index;
        return a.bytes === undefined ? 1 : -1;
    }
    return b.bytes - a.bytes || a.index - b.index;
}

export function groupEntries(entries: EntryInfo[], stats: StatsById): Group[] {
    return OWNERS.map((owner) => {
        const sorted = entries
            .filter((e) => ENTRY_META[e.id].owner === owner.id)
            .map((entry, index) => ({ entry, index, bytes: stats[entry.id]?.bytes }))
            .sort(bySizeThenOrder);
        return {
            owner,
            entries: sorted.map((r) => r.entry),
            bytes: sorted.reduce((sum, r) => sum + (r.bytes ?? 0), 0),
        };
    }).filter((g) => g.entries.length > 0);
}

export function reclaimable(entries: EntryInfo[], stats: StatsById): number {
    return entries.filter((e) => e.clearable && e.path).reduce((sum, e) => sum + (stats[e.id]?.bytes ?? 0), 0);
}

export function regenerableIds(entries: EntryInfo[], stats: StatsById): EntryId[] {
    return entries
        .filter(
            (e) => e.clearable && e.path && ENTRY_META[e.id].kind === "regenerates" && (stats[e.id]?.bytes ?? 0) > 0,
        )
        .map((e) => e.id);
}

export function sizeShare(bytes: number, total: number): number {
    return total > 0 ? bytes / total : 0;
}

const DAY_SECS = 86_400;

export function formatAge(thenSecs: number, nowSecs: number): string {
    const days = Math.max(0, Math.floor((nowSecs - thenSecs) / DAY_SECS));
    const plural = (n: number, unit: string) => `${n} ${unit}${n === 1 ? "" : "s"}`;
    if (days === 0) return "today";
    if (days < 30) return plural(days, "day");
    if (days < 365) return plural(Math.floor(days / 30), "month");
    return plural(Math.floor(days / 365), "year");
}

export function describeUnits(id: EntryId, stats: EntryStats, nowSecs: number): string | null {
    const noun = ENTRY_META[id].noun;
    if (!noun || !stats.count) return null;
    const count = `${stats.count} ${stats.count === 1 ? noun.one : noun.other}`;
    return stats.oldestSecs === null ? count : `${count}, oldest ${formatAge(stats.oldestSecs, nowSecs)}`;
}

export { storageClear, storageEntries, storageEntryStats, storageReveal } from "./api";
