import { describe, expect, it } from "vitest";
import { checkTiers } from "./tiers.mjs";

function pkg(name, tier, deps = [], dir = tier === undefined ? "ring0" : `ring${tier}`) {
    const app = name === "app";
    return {
        name,
        manifest_path: app ? "/repo/apps/desktop/src-tauri/Cargo.toml" : `/repo/crates/${dir}/${name}/Cargo.toml`,
        metadata: tier === undefined ? null : { dp: { tier } },
        dependencies: deps.map((d) => (typeof d === "string" ? { name: d, kind: null } : d)),
    };
}

const meta = (...packages) => ({ packages });

describe("crate tiers", () => {
    it("accepts a valid graph", () => {
        const graph = meta(pkg("dp-a", 0), pkg("dp-b", 1, ["dp-a", "serde"]), pkg("app", 3, ["dp-a", "dp-b", "tauri"]));
        expect(checkTiers(graph, [])).toEqual([]);
    });

    it("rejects an upward edge", () => {
        const graph = meta(pkg("dp-a", 0, ["dp-b"]), pkg("dp-b", 1));
        expect(checkTiers(graph, [])).toEqual([expect.stringContaining("dp-a (tier 0) depends on dp-b (tier 1)")]);
    });

    it("rejects a sideways edge outside the allowlist", () => {
        const graph = meta(pkg("dp-a", 0), pkg("dp-b", 0, ["dp-a"]));
        expect(checkTiers(graph, [])).toEqual([expect.stringContaining("dp-b (tier 0) depends on dp-a (tier 0)")]);
    });

    it("accepts a sideways edge in the allowlist", () => {
        const graph = meta(pkg("dp-a", 0), pkg("dp-b", 0, ["dp-a"]));
        expect(checkTiers(graph, [["dp-b", "dp-a"]])).toEqual([]);
    });

    it("rejects a stale allowlist entry", () => {
        const graph = meta(pkg("dp-a", 0), pkg("dp-b", 0));
        expect(checkTiers(graph, [["dp-b", "dp-a"]])).toEqual([
            expect.stringContaining("stale allowlist entry dp-b -> dp-a"),
        ]);
    });

    it("ignores dev-dependencies", () => {
        const graph = meta(pkg("dp-a", 0, [{ name: "dp-b", kind: "dev" }]), pkg("dp-b", 1));
        expect(checkTiers(graph, [])).toEqual([]);
    });

    it("checks build-dependencies", () => {
        const graph = meta(pkg("dp-a", 0, [{ name: "dp-b", kind: "build" }]), pkg("dp-b", 1));
        expect(checkTiers(graph, [])).toHaveLength(1);
    });

    it("rejects a crate without a tier", () => {
        expect(checkTiers(meta(pkg("dp-a", undefined)), [])).toEqual([
            expect.stringContaining("dp-a declares no tier"),
        ]);
    });

    it("rejects a tier that does not match the directory", () => {
        const graph = meta(pkg("dp-a", 1, [], "ring0"));
        expect(checkTiers(graph, [])).toEqual([
            expect.stringContaining("dp-a declares tier 1 but lives in crates/ring0"),
        ]);
    });

    it("rejects tauri below the app tier", () => {
        const graph = meta(pkg("dp-a", 0, ["tauri"]), pkg("dp-b", 1, ["tauri-plugin-log"]));
        expect(checkTiers(graph, [])).toEqual([
            expect.stringContaining("dp-a (tier 0) depends on tauri"),
            expect.stringContaining("dp-b (tier 1) depends on tauri-plugin-log"),
        ]);
    });
});
