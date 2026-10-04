import type { Hero } from "$lib/features/heroes/heroes";
import { t } from "$lib/core/i18n.svelte";
import type { DiscordClientKind } from "$lib/generated/types/DiscordClientKind";
import type { PresenceClient } from "$lib/generated/types/PresenceClient";
import type { PresenceLevelSetting } from "$lib/generated/types/PresenceLevelSetting";
import type { PresenceSettings } from "$lib/generated/types/PresenceSettings";
import type { PresenceStatus } from "$lib/generated/types/PresenceStatus";

export const ALL_CLIENT_KINDS: DiscordClientKind[] = ["stable", "ptb", "canary", "other"];
export const PRESENCE_LEVELS: PresenceLevelSetting[] = ["off", "basic", "detailed"];

export const DEFAULT_PRESENCE: PresenceSettings = { level: "off", clients: ALL_CLIENT_KINDS };

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === "object" && value !== null;
}

function canonical(kinds: Iterable<unknown>): DiscordClientKind[] {
    const set = new Set(kinds);
    return ALL_CLIENT_KINDS.filter((kind) => set.has(kind));
}

export function resolvePresence(raw: unknown): PresenceSettings {
    const stored = isRecord(raw) ? raw : {};
    const level = PRESENCE_LEVELS.find((l) => l === stored.level) ?? DEFAULT_PRESENCE.level;
    const clients = Array.isArray(stored.clients) ? canonical(stored.clients) : [...DEFAULT_PRESENCE.clients];
    return { level, clients };
}

export function toggleClient(clients: DiscordClientKind[], kind: DiscordClientKind, on: boolean): DiscordClientKind[] {
    return canonical(on ? [...clients, kind] : clients.filter((c) => c !== kind));
}

export function clientLabel(client: PresenceClient): string {
    switch (client.kind) {
        case "stable":
            return t("settings.discord.client_stable_name");
        case "ptb":
            return t("settings.discord.client_ptb_name");
        case "canary":
            return t("settings.discord.client_canary_name");
        case "other":
            return t("settings.discord.client_other_name", { pipe: client.pipeIndex });
    }
}

export function runningLine(enabled: boolean, status: PresenceStatus | null): string {
    if (!enabled) return t("settings.discord.status_off");
    if (!status) return "";
    const connected = status.clients.filter((c) => c.connected);
    if (connected.length === 0) return t("settings.discord.status_none");
    return t("settings.discord.status_showing", { clients: connected.map(clientLabel).join(", ") });
}

export function runningKinds(status: PresenceStatus | null): Set<DiscordClientKind> {
    return new Set(status?.clients.map((c) => c.kind));
}

export function heroNameMap(heroes: Record<number, Hero>): Record<number, string> {
    const out: Record<number, string> = {};
    for (const hero of Object.values(heroes)) out[hero.id] = hero.name;
    return out;
}
