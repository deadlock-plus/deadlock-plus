import { command, listen, type Unlisten } from "$lib/core/tauri";
import type { LiveState } from "$lib/generated/types/LiveState";

const SNAPSHOT_EVENT = "live-snapshot";

export type { LiveState };

export const getLiveState = () => command<LiveState>("get_live_state");
export const onLiveSnapshot = (handler: (state: LiveState) => void): Promise<Unlisten> =>
    listen<LiveState>(SNAPSHOT_EVENT, handler);
