import { invoke } from "@tauri-apps/api/core";

import type { AddonInfo } from "$lib/generated/types/AddonInfo";
import type { AddonListing } from "$lib/generated/types/AddonListing";
import type { AddonScan } from "$lib/generated/types/AddonScan";
import type { CaptureState } from "$lib/generated/types/CaptureState";
import type { CaptureStatus } from "$lib/generated/types/CaptureStatus";
import type { Finding } from "$lib/generated/types/Finding";
import type { FrameSpike } from "$lib/generated/types/FrameSpike";
import type { FrameStats } from "$lib/generated/types/FrameStats";
import type { Rule } from "$lib/generated/types/Rule";
import type { ScriptReport } from "$lib/generated/types/ScriptReport";
import type { Severity } from "$lib/generated/types/Severity";

export type {
    AddonInfo,
    AddonListing,
    AddonScan,
    CaptureState,
    CaptureStatus,
    Finding,
    FrameSpike,
    FrameStats,
    Rule,
    ScriptReport,
    Severity,
};

export function listAddons() {
    return invoke<AddonListing>("list_addons");
}

export function scanAddon(fileName: string) {
    return invoke<AddonScan>("scan_addon", { fileName });
}

export function startFrameCapture() {
    return invoke<void>("start_frame_capture");
}

export function frameCaptureStatus() {
    return invoke<CaptureStatus>("frame_capture_status");
}

export function stopFrameCapture() {
    return invoke<FrameStats>("stop_frame_capture");
}
