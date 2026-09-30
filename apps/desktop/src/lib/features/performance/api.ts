import { command } from "$lib/core/tauri";

import type { AddonInfo } from "$lib/generated/types/AddonInfo";
import type { AddonListing } from "$lib/generated/types/AddonListing";
import type { AddonScan } from "$lib/generated/types/AddonScan";
import type { AddonScanReport } from "$lib/generated/types/AddonScanReport";
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
    AddonScanReport,
    CaptureState,
    CaptureStatus,
    Finding,
    FrameSpike,
    FrameStats,
    Rule,
    ScriptReport,
    Severity,
};

/** `force` runs the scan even while Deadlock is running, ignoring the pause and slowdown. */
export function startAddonScan(force: boolean) {
    return command<void>("start_addon_scan", { force });
}

export function addonScanReport() {
    return command<AddonScanReport>("addon_scan_report");
}

export function startFrameCapture() {
    return command<void>("start_frame_capture");
}

export function frameCaptureStatus() {
    return command<CaptureStatus>("frame_capture_status");
}

export function stopFrameCapture() {
    return command<FrameStats>("stop_frame_capture");
}
