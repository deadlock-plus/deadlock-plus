import type { PingStats } from "$lib/generated/types/PingStats";
import type { RelayInfo } from "$lib/generated/types/RelayInfo";
import type { EndpointInfo } from "$lib/generated/types/EndpointInfo";
import type { NetworkSnapshot } from "$lib/generated/types/NetworkSnapshot";
import type { HistoryPoint } from "$lib/generated/types/HistoryPoint";
import type { PingSummary } from "$lib/generated/types/PingSummary";

export type { PingStats, RelayInfo, EndpointInfo, NetworkSnapshot, HistoryPoint, PingSummary };

export type NetworkPoll = { snapshot: NetworkSnapshot; tail: HistoryPoint[] };
