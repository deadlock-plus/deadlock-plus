import { kvGet, kvSet } from "$lib/kv";
import { getAppInfo } from "$lib/features/settings/about";
import { settings } from "$lib/features/settings/settings.svelte";
import { getChangelog } from "./changelog-source";
import { notesSince, parseChangelog, releasedUpTo, type ChangelogEntry } from "./changelog";

const STORE = "app-settings";
const LAST_SEEN_KEY = "lastSeenVersion";

class WhatsNew {
    entries = $state<ChangelogEntry[]>([]);
    history = $state<ChangelogEntry[]>([]);
    open = $state(false);

    async init() {
        await settings.ready;
        try {
            const { appVersion } = await getAppInfo();
            const lastSeen = await kvGet<string>(STORE, LAST_SEEN_KEY);
            const released = releasedUpTo(parseChangelog(await getChangelog()), appVersion);
            this.history = released;
            this.entries = notesSince(released, lastSeen, appVersion);
            this.open = this.entries.length > 0;
            if (lastSeen !== appVersion) await kvSet(STORE, LAST_SEEN_KEY, appVersion);
        } catch {
            // Not running inside Tauri, or the store is unavailable: skip the notes.
        }
    }

    close() {
        this.open = false;
    }
}

export const whatsNew = new WhatsNew();
