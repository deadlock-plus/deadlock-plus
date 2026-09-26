import { describe, expect, it } from "vitest";
import { compareVersions, notesSince, parseChangelog, releasedUpTo } from "./changelog";

const md = `# Changelog

Intro text.

## [Unreleased]

### Added

- Not shipped yet

## [0.2.0] - 2026-10-01

### Added

- Auto-update
- Tray icon

### Fixed

- Crash on start

## [0.1.1] - 2026-09-28

### Fixed

- Typo

## [0.1.0] - 2026-09-26

- First release
`;

describe("parseChangelog", () => {
    it("skips Unreleased and keeps release order", () => {
        expect(parseChangelog(md).map((e) => e.version)).toEqual(["0.2.0", "0.1.1", "0.1.0"]);
    });

    it("reads the date and grouped items", () => {
        const [latest] = parseChangelog(md);
        expect(latest.date).toBe("2026-10-01");
        expect(latest.sections).toEqual([
            { title: "Added", items: ["Auto-update", "Tray icon"] },
            { title: "Fixed", items: ["Crash on start"] },
        ]);
    });

    it("puts ungrouped bullets in a section without a title", () => {
        const last = parseChangelog(md).at(-1)!;
        expect(last.sections).toEqual([{ title: "", items: ["First release"] }]);
    });

    it("returns nothing for an empty log", () => {
        expect(parseChangelog("# Changelog\n\n## [Unreleased]\n")).toEqual([]);
    });
});

describe("compareVersions", () => {
    it("orders numerically, not lexically", () => {
        expect(compareVersions("0.10.0", "0.9.0")).toBeGreaterThan(0);
        expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
        expect(compareVersions("0.1.0", "0.1.1")).toBeLessThan(0);
    });

    it("ranks a prerelease below its release", () => {
        expect(compareVersions("1.0.0-beta.1", "1.0.0")).toBeLessThan(0);
    });
});

describe("releasedUpTo", () => {
    const entries = parseChangelog(md);

    it("drops entries newer than the running version", () => {
        expect(releasedUpTo(entries, "0.1.1").map((e) => e.version)).toEqual(["0.1.1", "0.1.0"]);
    });

    it("keeps everything when running the newest", () => {
        expect(releasedUpTo(entries, "0.2.0")).toHaveLength(3);
    });
});

describe("notesSince", () => {
    const entries = parseChangelog(md);

    it("returns releases newer than the last seen, up to the current one", () => {
        expect(notesSince(entries, "0.1.0", "0.2.0").map((e) => e.version)).toEqual(["0.2.0", "0.1.1"]);
    });

    it("stops at the running version", () => {
        expect(notesSince(entries, "0.1.0", "0.1.1").map((e) => e.version)).toEqual(["0.1.1"]);
    });

    it("shows nothing on a first launch", () => {
        expect(notesSince(entries, null, "0.2.0")).toEqual([]);
    });

    it("shows nothing when the version has not changed or went down", () => {
        expect(notesSince(entries, "0.2.0", "0.2.0")).toEqual([]);
        expect(notesSince(entries, "0.2.0", "0.1.0")).toEqual([]);
    });
});
