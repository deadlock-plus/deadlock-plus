import { describe, expect, it } from "vitest";
import { aboutGroups, aboutText, type AppInfo } from "./about";
import { LICENSES } from "./licenses";

const info = (over: Partial<AppInfo> = {}): AppInfo => ({
    appVersion: "0.1.0",
    tauriVersion: "2.11.6",
    webviewVersion: "141.0.1",
    os: "Windows 11 Pro 26200",
    arch: "x86_64",
    debugBuild: false,
    elevated: true,
    dataDir: "C:\data",
    gameDir: "D:\Deadlock",
    gameBuild: {
        clientVersion: "6701",
        sourceRevision: "11038876",
        versionDate: "Sep 25 2026",
        versionTime: "11:10:43",
    },
    ...over,
});

const flat = (i: AppInfo) => aboutGroups(i).flatMap((g) => g.rows);

describe("aboutGroups", () => {
    it("reports app, runtime, elevation and game build", () => {
        const rows = Object.fromEntries(flat(info()));
        expect(rows["Version"]).toBe("0.1.0");
        expect(rows["Tauri"]).toBe("2.11.6");
        expect(rows["WebView2"]).toBe("141.0.1");
        expect(rows["Running as"]).toBe("Administrator");
        expect(rows["Game build"]).toBe("6701");
        expect(rows["Game built"]).toBe("Sep 25 2026 11:10:43");
    });

    it("marks dev builds and non-elevated runs", () => {
        const rows = Object.fromEntries(flat(info({ debugBuild: true, elevated: false })));
        expect(rows["Version"]).toBe("0.1.0 (dev build)");
        expect(rows["Running as"]).toBe("Standard user");
    });

    it("drops rows it has no data for and hides the game group when Deadlock is missing", () => {
        const groups = aboutGroups(info({ webviewVersion: null, gameDir: null, gameBuild: null }));
        expect(flat(info({ webviewVersion: null })).some(([k]) => k === "WebView2")).toBe(false);
        expect(groups.map((g) => g.title)).toEqual(["Deadlock+"]);
    });

    it("shows the install folder even when steam.inf could not be read", () => {
        const groups = aboutGroups(info({ gameBuild: null }));
        expect(groups.find((g) => g.title === "Deadlock")?.rows).toEqual([["Install folder", "D:\Deadlock"]]);
    });
});

describe("aboutText", () => {
    it("is plain text with one 'label: value' line per row", () => {
        const text = aboutText(info());
        expect(text).toContain("Version: 0.1.0");
        expect(text).toContain("Game build: 6701");
        expect(text.split("\n").every((l) => l === "" || l.includes(": ") || !l.startsWith(" "))).toBe(true);
    });
});

describe("LICENSES", () => {
    it("has an owner and a notice for every entry, and covers the bundled font families", () => {
        for (const l of LICENSES) {
            expect(l.name.length).toBeGreaterThan(0);
            expect(l.owner.length).toBeGreaterThan(0);
            expect(l.notice.length).toBeGreaterThan(0);
        }
        const names = LICENSES.map((l) => l.name);
        for (const family of ["Retail Demo", "Valve Oracle", "Valve Pulp", "Atkinson Hyperlegible", "Limelight"]) {
            expect(names).toContain(family);
        }
    });
});
