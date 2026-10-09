import { describe, expect, it } from "vitest";
import { assetName, changelogNotes, updaterManifest } from "./release.mjs";

const md = `# Changelog

## [Unreleased]

- Later

## [0.2.0] - 2026-10-01

### Added

- Auto-update

## [0.1.0] - 2026-09-26

- First release
`;

describe("changelogNotes", () => {
    it("returns the body of one release", () => {
        expect(changelogNotes(md, "0.2.0")).toBe("### Added\n\n- Auto-update");
    });

    it("stops at the next release heading", () => {
        expect(changelogNotes(md, "0.1.0")).toBe("- First release");
    });

    it("returns null for a version that has no entry", () => {
        expect(changelogNotes(md, "0.3.0")).toBeNull();
    });

    it("returns null for an empty entry", () => {
        expect(changelogNotes("## [1.0.0] - 2026-01-01\n\n## [0.9.0] - 2025-01-01\n\n- x\n", "1.0.0")).toBeNull();
    });
});

describe("assetName", () => {
    it("has no characters GitHub rewrites", () => {
        expect(assetName("0.2.0", "windows-x86_64")).toBe("Deadlock-Plus_0.2.0_x64-setup.exe");
    });

    it("names the Linux bundles per package format", () => {
        expect(assetName("0.2.0", "linux-x86_64-appimage")).toBe("Deadlock-Plus_0.2.0_amd64.AppImage");
        expect(assetName("0.2.0", "linux-x86_64-deb")).toBe("Deadlock-Plus_0.2.0_amd64.deb");
    });

    it("rejects an unknown platform", () => {
        expect(() => assetName("0.2.0", "macos-aarch64")).toThrow();
    });
});

describe("updaterManifest", () => {
    const manifest = updaterManifest({
        version: "0.2.0",
        notes: "### Added\n\n- Auto-update",
        pubDate: "2026-10-01T12:00:00Z",
        signatures: { "windows-x86_64": "c2ln", "linux-x86_64-appimage": "YXBw", "linux-x86_64-deb": "ZGVi" },
        repo: "deadlock-plus/deadlock-plus",
    });
    const base = "https://github.com/deadlock-plus/deadlock-plus/releases/download/v0.2.0";

    it("carries version, notes and date", () => {
        expect(manifest.version).toBe("0.2.0");
        expect(manifest.notes).toBe("### Added\n\n- Auto-update");
        expect(manifest.pub_date).toBe("2026-10-01T12:00:00Z");
    });

    it("points the windows platform at the tagged release asset", () => {
        expect(manifest.platforms["windows-x86_64"]).toEqual({
            signature: "c2ln",
            url: `${base}/Deadlock-Plus_0.2.0_x64-setup.exe`,
        });
    });

    it("lists one Linux entry per package format", () => {
        expect(manifest.platforms["linux-x86_64-appimage"]).toEqual({
            signature: "YXBw",
            url: `${base}/Deadlock-Plus_0.2.0_amd64.AppImage`,
        });
        expect(manifest.platforms["linux-x86_64-deb"]).toEqual({
            signature: "ZGVi",
            url: `${base}/Deadlock-Plus_0.2.0_amd64.deb`,
        });
    });

    it("refuses to build a feed with a platform missing its signature", () => {
        expect(() =>
            updaterManifest({
                version: "0.2.0",
                notes: "x",
                pubDate: "2026-10-01T12:00:00Z",
                signatures: { "windows-x86_64": "c2ln" },
                repo: "deadlock-plus/deadlock-plus",
            }),
        ).toThrow(/linux-x86_64-appimage/);
    });
});
