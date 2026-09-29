import { describe, expect, it } from "vitest";
import data from "$lib/generated/dependency-licenses.json";
import { summarize, type DependencyLicenses } from "./dependencies";

const deps = data as DependencyLicenses;

describe("generated dependency licences", () => {
    it("lists both ecosystems and the direct dependencies", () => {
        const pkgs = deps.groups.flatMap((g) => g.packages);
        for (const [source, name] of [
            ["rust", "tauri"],
            ["rust", "reqwest"],
            ["rust", "steamlocate"],
            ["npm", "svelte"],
            ["npm", "bits-ui"],
            ["npm", "@lucide/svelte"],
        ] as const) {
            expect(
                pkgs.some((p) => p.source === source && p.name === name),
                `${source}:${name}`,
            ).toBe(true);
        }
    });

    it("keeps build and dev tools out", () => {
        const names = new Set(deps.groups.flatMap((g) => g.packages.map((p) => p.name)));
        for (const dev of ["tauri-build", "vitest", "vite", "typescript", "@tauri-apps/cli"]) {
            expect(names.has(dev), dev).toBe(false);
        }
    });

    it("gives every group a title and packages, and non-empty text when it has any", () => {
        for (const g of deps.groups) {
            expect(g.title.length).toBeGreaterThan(0);
            expect(g.packages.length).toBeGreaterThan(0);
            if (g.text !== null) expect(g.text.trim().length).toBeGreaterThan(0);
        }
    });

    it("has no duplicate package within a group", () => {
        for (const g of deps.groups) {
            const keys = g.packages.map((p) => `${p.source}:${p.name}@${p.version}`);
            expect(new Set(keys).size).toBe(keys.length);
        }
    });
});

describe("summarize", () => {
    it("counts distinct packages once even when they appear in several groups", () => {
        const pkg = (name: string, source: "rust" | "npm" = "rust") => ({ name, version: "1", source });
        const s = summarize({
            groups: [
                { title: "MIT", text: "a", packages: [pkg("x"), pkg("y")] },
                { title: "Apache-2.0", text: "b", packages: [pkg("x"), pkg("z", "npm")] },
            ],
        });
        expect(s).toEqual({ packages: 3, rust: 2, npm: 1, groups: 2 });
    });
});
