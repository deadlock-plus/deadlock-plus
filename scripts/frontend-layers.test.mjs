import { describe, expect, it } from "vitest";
import { checkLayers, extractImports } from "./frontend-layers.mjs";

const f = (file, ...imports) => ({ file, imports });

describe("frontend layers", () => {
    it("accepts a clean tree", () => {
        const files = [
            f("lib/core/ipc.ts", "@tauri-apps/api/core"),
            f("lib/features/demos/api.ts", "@tauri-apps/api/core", "$lib/core/ipc"),
            f("lib/features/demos/demos.ts", "./api"),
            f("lib/ui/button.svelte", "$lib/core/utils"),
            f("lib/shell/sidebar.svelte", "$lib/features/registry"),
        ];
        expect(checkLayers(files, [])).toEqual([]);
    });

    it("rejects tauri outside core and feature api files", () => {
        const files = [
            f("lib/features/demos/demos.ts", "@tauri-apps/api/core"),
            f("routes/+page.svelte", "@tauri-apps/plugin-log"),
        ];
        expect(checkLayers(files, [])).toEqual([
            expect.stringContaining("lib/features/demos/demos.ts imports @tauri-apps/api/core"),
            expect.stringContaining("routes/+page.svelte imports @tauri-apps/plugin-log"),
        ]);
    });

    it("exempts test files from the tauri rule", () => {
        expect(checkLayers([f("lib/features/demos/demos.test.ts", "@tauri-apps/api/core")], [])).toEqual([]);
    });

    it("rejects core and ui importing features, absolute or relative", () => {
        const files = [
            f("lib/core/a.ts", "$lib/features/demos/demos"),
            f("lib/ui/b.svelte", "../features/demos/demos"),
        ];
        expect(checkLayers(files, [])).toEqual([
            expect.stringContaining("lib/core/a.ts imports $lib/features/demos/demos"),
            expect.stringContaining("lib/ui/b.svelte imports ../features/demos/demos"),
        ]);
    });

    it("lets shell import features only through the registry", () => {
        const files = [
            f("lib/shell/a.svelte", "$lib/features/registry"),
            f("lib/shell/b.svelte", "$lib/features/home/home"),
        ];
        expect(checkLayers(files, [])).toEqual([
            expect.stringContaining("lib/shell/b.svelte imports $lib/features/home/home"),
        ]);
    });

    it("suppresses violations listed in the allow-list", () => {
        const files = [f("lib/features/demos/demos.ts", "@tauri-apps/api/core")];
        expect(checkLayers(files, [["lib/features/demos/demos.ts", "tauri"]])).toEqual([]);
    });

    it("keeps an allow-list entry scoped to its rule", () => {
        const files = [f("lib/core/a.ts", "$lib/features/x/y")];
        expect(checkLayers(files, [["lib/core/a.ts", "tauri"]])).toEqual([
            expect.stringContaining("lib/core/a.ts imports $lib/features/x/y"),
            expect.stringContaining("stale allow-list entry lib/core/a.ts (tauri)"),
        ]);
    });

    it("rejects a stale allow-list entry", () => {
        const files = [f("lib/features/demos/api.ts", "@tauri-apps/api/core")];
        expect(checkLayers(files, [["lib/features/demos/api.ts", "tauri"]])).toEqual([
            expect.stringContaining("stale allow-list entry lib/features/demos/api.ts (tauri)"),
        ]);
    });

    it("extracts static, side-effect, re-export and dynamic imports", () => {
        const src = `
            import a from "./a";
            import type { B } from '$lib/b';
            import "./side";
            export { c } from "./c";
            const d = await import("./d");
            const s = "not from an import";
        `;
        expect(extractImports(src)).toEqual(["./a", "$lib/b", "./side", "./c", "./d"]);
    });
});
