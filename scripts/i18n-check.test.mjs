import { describe, expect, it } from "vitest";
import { checkCatalogs, extractKeys, flatten } from "./i18n-check.mjs";

describe("flatten", () => {
    it("joins nested keys with dots", () => {
        expect(flatten({ a: { b: "x", c: { d: "y" } }, e: "z" })).toEqual({ "a.b": "x", "a.c.d": "y", e: "z" });
    });
});

describe("extractKeys", () => {
    it("finds literal t and tn keys", () => {
        const source = `t("common.save"); tn('replays_count', n); foo.t("x.y", {a: 1}); format("nope")`;
        expect(extractKeys(source)).toEqual([
            { key: "common.save", plural: false },
            { key: "replays_count", plural: true },
            { key: "x.y", plural: false },
        ]);
    });

    it("ignores dynamic keys", () => {
        expect(extractKeys("t(`a.${b}`); t(key)")).toEqual([]);
    });
});

describe("checkCatalogs", () => {
    const en = { common: { save: "Save", hi: "Hi {name}" }, n_one: "{count} a", n_other: "{count} as" };

    it("accepts a clean setup", () => {
        const used = [
            { file: "a.ts", key: "common.save", plural: false },
            { file: "a.ts", key: "n", plural: true },
        ];
        expect(checkCatalogs({ en }, used).errors).toEqual([]);
    });

    it("reports a used key missing from en", () => {
        const errors = checkCatalogs({ en }, [{ file: "a.ts", key: "common.gone", plural: false }]).errors;
        expect(errors).toEqual([expect.stringContaining("a.ts uses missing key common.gone")]);
    });

    it("reports a plural base without an _other form", () => {
        const errors = checkCatalogs({ en: { n_one: "{count} a" } }, [{ file: "a.ts", key: "n", plural: true }]).errors;
        expect(errors).toEqual([expect.stringContaining("n_other")]);
    });

    it("reports placeholder mismatches in other locales", () => {
        const fr = { common: { save: "Enregistrer", hi: "Salut" } };
        const errors = checkCatalogs({ en, fr }, []).errors;
        expect(errors).toEqual([expect.stringContaining("fr: common.hi placeholders")]);
    });

    it("reports an invalid plural suffix in other locales", () => {
        const fr = { n_few_other: "x", n_bad: "{count}" };
        const errors = checkCatalogs({ en, fr }, []).errors;
        expect(errors.some((e) => e.includes("fr: n_bad"))).toBe(true);
    });

    it("warns, not errors, on unused en keys", () => {
        const result = checkCatalogs({ en }, []);
        expect(result.errors).toEqual([]);
        expect(result.warnings.length).toBeGreaterThan(0);
    });
});
