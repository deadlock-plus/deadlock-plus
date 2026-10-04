import { describe, expect, it } from "vitest";
import { SUPPORTED_LOCALES } from "$lib/core/i18n.svelte";
import { LANGUAGES, completion, languageOptions } from "./languages";

const reference = {
    a: "One",
    nested: { b: "Two", c: "Three" },
    count_one: "{count} item",
    count_other: "{count} items",
};

describe("completion", () => {
    it("is 0 for a missing or empty catalog", () => {
        expect(completion(undefined, reference)).toBe(0);
        expect(completion({}, reference)).toBe(0);
    });

    it("is 100 for a full copy", () => {
        expect(completion(reference, reference)).toBe(100);
    });

    it("counts the share of translated keys, rounded down", () => {
        expect(completion({ a: "Un", nested: { b: "Deux" } }, reference)).toBe(50);
        expect(completion({ a: "Un" }, reference)).toBe(25);
    });

    it("ignores blank strings and keys that are not in the reference", () => {
        expect(completion({ a: "  ", extra: "x" }, reference)).toBe(0);
    });

    it("counts a plural group once, however many forms it has", () => {
        const many = { count_one: "a", count_few: "b", count_many: "c", count_other: "d" };
        expect(completion(many, reference)).toBe(25);
    });

    it("never reports 100 while a key is still missing", () => {
        const big = Object.fromEntries(Array.from({ length: 1000 }, (_, i) => [`k${i}`, "x"]));
        const nearly = { ...big };
        delete nearly.k0;
        expect(completion(nearly, big)).toBe(99);
    });
});

describe("LANGUAGES", () => {
    it("has unique codes, English first", () => {
        expect(LANGUAGES[0].code).toBe("en");
        expect(new Set(LANGUAGES.map((l) => l.code)).size).toBe(LANGUAGES.length);
    });

    it("covers every language Crowdin targets", () => {
        const codes = LANGUAGES.map((l) => l.code);
        for (const code of [
            "af",
            "cs",
            "de",
            "es",
            "fr",
            "hu",
            "id",
            "it",
            "ja",
            "ko",
            "pl",
            "pt",
            "pt-BR",
            "ru",
            "th",
            "tr",
            "uk",
            "zh-CN",
            "zh-TW",
        ])
            expect(codes).toContain(code);
    });
});

describe("shipped locales", () => {
    it("registers a loader for every listed language", () => {
        for (const language of LANGUAGES) expect(SUPPORTED_LOCALES).toContain(language.code);
    });
});

describe("languageOptions", () => {
    const catalogs = { en: reference, fr: { a: "Un" } };

    it("marks only supported locales as selectable", () => {
        const options = languageOptions(["en", "fr"], catalogs, reference);
        expect(options.find((o) => o.code === "en")?.available).toBe(true);
        expect(options.find((o) => o.code === "fr")?.available).toBe(true);
        expect(options.find((o) => o.code === "de")?.available).toBe(false);
    });

    it("reports completion from the catalogs, with English always complete", () => {
        const options = languageOptions(["en", "fr"], catalogs, reference);
        expect(options.find((o) => o.code === "en")?.completion).toBe(100);
        expect(options.find((o) => o.code === "fr")?.completion).toBe(25);
        expect(options.find((o) => o.code === "de")?.completion).toBe(0);
    });

    it("lists a supported locale that has no entry, such as the dev pseudo-locale", () => {
        const options = languageOptions(["en", "en-XA"], { en: reference, "en-XA": reference }, reference);
        const pseudo = options.find((o) => o.code === "en-XA");
        expect(pseudo?.available).toBe(true);
        expect(pseudo?.flag).toBeNull();
        expect(pseudo?.completion).toBe(100);
    });

    it("sorts selectable languages first, then by completion", () => {
        const options = languageOptions(["en", "fr"], catalogs, reference);
        expect(options.slice(0, 2).map((o) => o.code)).toEqual(["en", "fr"]);
    });
});
