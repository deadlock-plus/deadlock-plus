import type { AddonInfo, AddonScan, Finding, Rule, Severity } from "./api";

export type ScanStatus = "noScripts" | "clean" | "flagged";

export type LocatedFinding = Finding & { path: string };

const SEVERITY_RANK: Record<Severity, number> = { low: 0, medium: 1, high: 2 };

export const RULE_TITLES: Record<Rule, string> = {
    nulledNotCancelled: "Timer handle cleared without cancelling",
    unguardedRearm: "Timer re-arms itself with no cancel",
};

export function findingCount(scan: AddonScan): number {
    return scan.scripts.reduce((n, s) => n + s.findings.length, 0);
}

export function scanStatus(scan: AddonScan): ScanStatus {
    if (scan.scriptsScanned === 0) return "noScripts";
    return findingCount(scan) > 0 ? "flagged" : "clean";
}

export function worstSeverity(scan: AddonScan): Severity | null {
    let worst: Severity | null = null;
    for (const script of scan.scripts) {
        for (const f of script.findings) {
            if (worst === null || SEVERITY_RANK[f.severity] > SEVERITY_RANK[worst]) worst = f.severity;
        }
    }
    return worst;
}

export function flattenFindings(scan: AddonScan): LocatedFinding[] {
    return scan.scripts
        .flatMap((s) => s.findings.map((f) => ({ ...f, path: s.path })))
        .sort(
            (a, b) =>
                SEVERITY_RANK[b.severity] - SEVERITY_RANK[a.severity] ||
                a.path.localeCompare(b.path) ||
                a.line - b.line,
        );
}

export function addonTitle(info: AddonInfo, scan: AddonScan | undefined): string {
    if (scan) return scan.label;
    if (info.modId) return `GameBanana mod ${info.modId}`;
    return info.fileName;
}

export interface ScanSummary {
    scanned: number;
    noScripts: number;
    clean: number;
    flagged: number;
    findings: number;
}

export function summarize(scans: Record<string, AddonScan>): ScanSummary {
    const summary: ScanSummary = { scanned: 0, noScripts: 0, clean: 0, flagged: 0, findings: 0 };
    for (const scan of Object.values(scans)) {
        summary.scanned++;
        summary.findings += findingCount(scan);
        summary[scanStatus(scan)]++;
    }
    return summary;
}

export interface AddonGroups {
    flagged: AddonInfo[];
    clean: AddonInfo[];
    noScripts: AddonInfo[];
    failed: AddonInfo[];
    pending: AddonInfo[];
}

export function groupAddons(
    addons: AddonInfo[],
    scans: Record<string, AddonScan>,
    failures: Record<string, string>,
): AddonGroups {
    const groups: AddonGroups = { flagged: [], clean: [], noScripts: [], failed: [], pending: [] };
    for (const addon of addons) {
        const scan = scans[addon.fileName];
        if (failures[addon.fileName]) groups.failed.push(addon);
        else if (!scan) groups.pending.push(addon);
        else groups[scanStatus(scan)].push(addon);
    }
    const weight = (a: AddonInfo) => {
        const scan = scans[a.fileName];
        return { severity: SEVERITY_RANK[worstSeverity(scan) ?? "low"], count: findingCount(scan) };
    };
    groups.flagged.sort((a, b) => {
        const [x, y] = [weight(a), weight(b)];
        return y.severity - x.severity || y.count - x.count;
    });
    return groups;
}

export function scanPercent(done: number, total: number): number {
    return total > 0 ? Math.min(100, Math.round((done / total) * 100)) : 0;
}
