import { invoke } from "@tauri-apps/api/core";

import type { AddonInfo } from "$lib/generated/types/AddonInfo";
import type { AddonListing } from "$lib/generated/types/AddonListing";
import type { AddonScan } from "$lib/generated/types/AddonScan";
import type { Finding } from "$lib/generated/types/Finding";
import type { Rule } from "$lib/generated/types/Rule";
import type { ScriptReport } from "$lib/generated/types/ScriptReport";
import type { Severity } from "$lib/generated/types/Severity";

export type { AddonInfo, AddonListing, AddonScan, Finding, Rule, ScriptReport, Severity };

export function listAddons() {
    return invoke<AddonListing>("list_addons");
}

export function scanAddon(fileName: string) {
    return invoke<AddonScan>("scan_addon", { fileName });
}
