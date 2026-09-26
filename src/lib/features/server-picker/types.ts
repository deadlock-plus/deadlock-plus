import type { KeywordFilterMode } from "$lib/generated/types/KeywordFilterMode";
import type { RoutingNoteDefinition } from "$lib/generated/types/RoutingNoteDefinition";
import type { GameDefinition } from "$lib/generated/types/GameDefinition";
import type { RoutingNoteInfo } from "$lib/generated/types/RoutingNoteInfo";
import type { ServerGroup } from "$lib/generated/types/ServerGroup";
import type { ServerData } from "$lib/generated/types/ServerData";
import type { ExternalScan } from "$lib/generated/types/ExternalScan";
import type { FirewallCapability } from "$lib/generated/types/FirewallCapability";

export type {
    KeywordFilterMode,
    RoutingNoteDefinition,
    GameDefinition,
    RoutingNoteInfo,
    ServerGroup,
    ServerData,
    ExternalScan,
    FirewallCapability,
};
export type PingResults = Record<string, number | null>;
