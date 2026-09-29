import { listAddons, scanAddon, type AddonListing, type AddonScan } from "./api";
import { settings } from "$lib/features/settings/settings.svelte";
import { groupAddons, summarize } from "./performance";

const AUTO_SCAN_DELAY_MS = 8000;

class PerformanceScanStore {
    listing = $state<AddonListing | null>(null);
    scans = $state<Record<string, AddonScan>>({});
    failures = $state<Record<string, string>>({});
    error = $state<string | null>(null);
    scanning = $state(false);
    current = $state<string | null>(null);
    done = $state(0);
    hasRun = $state(false);

    addons = $derived(this.listing?.addons ?? []);
    summary = $derived(summarize(this.scans));
    groups = $derived(groupAddons(this.addons, this.scans, this.failures));

    async run() {
        if (this.scanning) return;
        this.scanning = true;
        this.hasRun = true;
        this.done = 0;
        this.current = null;
        this.scans = {};
        this.failures = {};
        this.error = null;
        try {
            this.listing = await listAddons();
            for (const addon of this.listing.addons) {
                this.current = addon.fileName;
                try {
                    this.scans[addon.fileName] = await scanAddon(addon.fileName);
                } catch (e) {
                    this.failures[addon.fileName] = String(e);
                }
                this.done++;
            }
        } catch (e) {
            this.error = String(e);
        } finally {
            this.scanning = false;
            this.current = null;
        }
    }

    start() {
        const timer = setTimeout(async () => {
            await settings.ready;
            if (settings.autoScanAddons && !this.hasRun) void this.run();
        }, AUTO_SCAN_DELAY_MS);
        return () => clearTimeout(timer);
    }
}

export const performanceScan = new PerformanceScanStore();
