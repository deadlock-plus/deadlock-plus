import { describe, expect, it } from "vitest";
import { pseudoLocalize, pseudoText } from "./i18n-pseudo";
import type { Catalog } from "./i18n.svelte";

describe("pseudoText", () => {
    it("wraps in brackets and accents letters", () => {
        const out = pseudoText("Save");
        expect(out.startsWith("[")).toBe(true);
        expect(out.endsWith("]")).toBe(true);
        expect(out).toContain("Šåṽé");
    });

    it("keeps placeholders untouched", () => {
        const out = pseudoText("Hello, {name}. You have {count} items.");
        expect(out).toContain("{name}");
        expect(out).toContain("{count}");
    });

    it("pads the text to expose overflow", () => {
        const source = "Delete this demo";
        expect(pseudoText(source).length).toBeGreaterThan(source.length * 1.3);
    });
});

describe("pseudoLocalize", () => {
    const catalog: Catalog = {
        common: { save: "Save", hello: "Hello, {name}.", nested: { deep: "Deep" } },
        replays_count_one: "{count} replay",
        replays_count_other: "{count} replays",
    };

    it("preserves keys, including plural suffixes and nesting", () => {
        const out = pseudoLocalize(catalog);
        expect(Object.keys(out).sort()).toEqual(Object.keys(catalog).sort());
        expect(Object.keys(out.common as Catalog).sort()).toEqual(["hello", "nested", "save"]);
        expect(Object.keys((out.common as Catalog).nested as Catalog)).toEqual(["deep"]);
    });

    it("transforms every leaf and keeps placeholder sets", () => {
        const out = pseudoLocalize(catalog);
        expect(out.replays_count_one).toMatch(/^\[.*\{count\}.*\]$/);
        expect((out.common as Catalog).hello).toContain("{name}");
    });

    it("does not mutate the input", () => {
        pseudoLocalize(catalog);
        expect((catalog.common as Catalog).save).toBe("Save");
    });
});
