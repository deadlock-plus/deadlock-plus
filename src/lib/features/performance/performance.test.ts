import { describe, expect, it } from "vitest";
import type { AddonInfo, AddonScan, Finding } from "./api";
import {
    addonTitle,
    findingCount,
    groupAddons,
    scanPercent,
    flattenFindings,
    scanStatus,
    summarize,
    worstSeverity,
} from "./performance";

const finding = (over: Partial<Finding> = {}): Finding => ({
    rule: "nulledNotCancelled",
    severity: "medium",
    function: "Update",
    line: 10,
    snippet: "handle = null;",
    message: "msg",
    ...over,
});

const scan = (over: Partial<AddonScan> = {}): AddonScan => ({
    fileName: "pak01_dir.vpk",
    label: "build_qollock",
    scriptsScanned: 3,
    scripts: [],
    ...over,
});

describe("scanStatus", () => {
    it("reports no scripts when none were scanned", () => {
        expect(scanStatus(scan({ scriptsScanned: 0 }))).toBe("noScripts");
    });

    it("reports clean when scripts were scanned without findings", () => {
        expect(scanStatus(scan({ scripts: [{ path: "a.js", findings: [] }] }))).toBe("clean");
    });

    it("reports flagged when any script has a finding", () => {
        expect(scanStatus(scan({ scripts: [{ path: "a.js", findings: [finding()] }] }))).toBe("flagged");
    });
});

describe("flattenFindings", () => {
    it("sorts high before medium before low, then by path and line", () => {
        const s = scan({
            scripts: [
                { path: "b.js", findings: [finding({ severity: "low", line: 1 })] },
                {
                    path: "a.js",
                    findings: [finding({ severity: "high", line: 9 }), finding({ severity: "medium", line: 2 })],
                },
                { path: "a.js", findings: [finding({ severity: "high", line: 3 })] },
            ],
        });
        expect(flattenFindings(s).map((f) => [f.path, f.severity, f.line])).toEqual([
            ["a.js", "high", 3],
            ["a.js", "high", 9],
            ["a.js", "medium", 2],
            ["b.js", "low", 1],
        ]);
    });
});

describe("worstSeverity and findingCount", () => {
    const s = scan({
        scripts: [
            { path: "a.js", findings: [finding({ severity: "low" }), finding({ severity: "medium" })] },
            { path: "b.js", findings: [finding({ severity: "low" })] },
        ],
    });

    it("picks the highest severity", () => {
        expect(worstSeverity(s)).toBe("medium");
    });

    it("returns null with no findings", () => {
        expect(worstSeverity(scan())).toBeNull();
    });

    it("counts findings across scripts", () => {
        expect(findingCount(s)).toBe(3);
    });
});

describe("addonTitle", () => {
    const info = (over: Partial<AddonInfo> = {}): AddonInfo => ({
        fileName: "pak01_dir.vpk",
        modId: null,
        enabled: null,
        ...over,
    });

    it("uses the scan label once scanned", () => {
        expect(addonTitle(info(), scan({ label: "build_qollock" }))).toBe("build_qollock");
    });

    it("falls back to the mod id before a scan", () => {
        expect(addonTitle(info({ modId: "123" }), undefined)).toBe("GameBanana mod 123");
    });

    it("falls back to the file name with nothing else", () => {
        expect(addonTitle(info(), undefined)).toBe("pak01_dir.vpk");
    });
});

describe("summarize", () => {
    it("counts addons by outcome", () => {
        const scans = {
            a: scan({ scriptsScanned: 0 }),
            b: scan({ scripts: [{ path: "x.js", findings: [] }] }),
            c: scan({ scripts: [{ path: "x.js", findings: [finding(), finding()] }] }),
        };
        expect(summarize(scans)).toEqual({ scanned: 3, noScripts: 1, clean: 1, flagged: 1, findings: 2 });
    });
});

describe("groupAddons", () => {
    const info = (fileName: string): AddonInfo => ({ fileName, modId: null, enabled: null });
    const flagged = (...severities: Finding["severity"][]) =>
        scan({ scripts: [{ path: "a.js", findings: severities.map((severity) => finding({ severity })) }] });

    it("splits addons by outcome and orders flagged worst first", () => {
        const addons = ["low", "high", "medium", "clean", "none", "bad", "wait"].map(info);
        const groups = groupAddons(
            addons,
            {
                low: flagged("low"),
                high: flagged("high"),
                medium: flagged("medium", "medium"),
                clean: scan({ scripts: [{ path: "a.js", findings: [] }] }),
                none: scan({ scriptsScanned: 0 }),
            },
            { bad: "boom" },
        );
        expect(groups.flagged.map((a) => a.fileName)).toEqual(["high", "medium", "low"]);
        expect(groups.clean.map((a) => a.fileName)).toEqual(["clean"]);
        expect(groups.noScripts.map((a) => a.fileName)).toEqual(["none"]);
        expect(groups.failed.map((a) => a.fileName)).toEqual(["bad"]);
        expect(groups.pending.map((a) => a.fileName)).toEqual(["wait"]);
    });

    it("breaks severity ties by finding count", () => {
        const groups = groupAddons(
            [info("one"), info("two")],
            { one: flagged("high"), two: flagged("high", "low") },
            {},
        );
        expect(groups.flagged.map((a) => a.fileName)).toEqual(["two", "one"]);
    });
});

describe("scanPercent", () => {
    it("rounds done over total", () => {
        expect(scanPercent(1, 3)).toBe(33);
    });

    it("is 0 with nothing to scan", () => {
        expect(scanPercent(0, 0)).toBe(0);
    });

    it("never exceeds 100", () => {
        expect(scanPercent(5, 3)).toBe(100);
    });
});
