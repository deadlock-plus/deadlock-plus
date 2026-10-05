import { describe, expect, it } from "vitest";
import { appLanguage, contributorsFrom, mergeThanks, translatorsFrom } from "./gen-thanks.mjs";

describe("contributorsFrom", () => {
    it("lists people by most commits and skips bots and anyone with none", () => {
        const people = contributorsFrom([
            { login: "ada", type: "User", contributions: 3, html_url: "https://github.com/ada" },
            {
                login: "dependabot[bot]",
                type: "Bot",
                contributions: 50,
                html_url: "https://github.com/apps/dependabot",
            },
            { login: "bo", type: "User", contributions: 9, html_url: "https://github.com/bo" },
            { login: "ghost", type: "User", contributions: 0, html_url: "https://github.com/ghost" },
        ]);
        expect(people).toEqual([
            { name: "bo", url: "https://github.com/bo" },
            { name: "ada", url: "https://github.com/ada" },
        ]);
    });

    it("skips the Crowdin sync account by default", () => {
        const rows = [{ login: "crowdin-bot", type: "User", contributions: 4, html_url: "u" }];
        expect(contributorsFrom(rows)).toEqual([]);
    });

    it("skips accounts named in the exclude list, ignoring case", () => {
        const people = contributorsFrom([{ login: "Ada", type: "User", contributions: 1, html_url: "u" }], ["ada"]);
        expect(people).toEqual([]);
    });
});

describe("appLanguage", () => {
    it("maps Crowdin ids to the codes the app loads", () => {
        expect(appLanguage("es-ES")).toBe("es");
        expect(appLanguage("pt-PT")).toBe("pt");
        expect(appLanguage("pt-BR")).toBe("pt-BR");
        expect(appLanguage("de")).toBe("de");
    });

    it("returns null for a language the app does not offer", () => {
        expect(appLanguage("xx")).toBeNull();
    });
});

describe("translatorsFrom", () => {
    const row = (username, translated, approved, languages) => ({
        user: { username, fullName: "Real Name" },
        translated,
        approved,
        languages: languages.map((id) => ({ id })),
    });

    it("uses the public username, links the profile and orders by words done", () => {
        const people = translatorsFrom([row("ada", 10, 0, ["de"]), row("bo", 100, 20, ["fr", "es-ES"])]);
        expect(people).toEqual([
            { name: "bo", url: "https://crowdin.com/profile/bo", languages: ["fr", "es"] },
            { name: "ada", url: "https://crowdin.com/profile/ada", languages: ["de"] },
        ]);
    });

    it("skips people who did no work and languages the app does not offer", () => {
        const people = translatorsFrom([row("idle", 0, 0, ["de"]), row("ada", 5, 0, ["de", "xx"])]);
        expect(people.map((p) => [p.name, p.languages])).toEqual([["ada", ["de"]]]);
    });

    it("drops a translator whose every language is unknown", () => {
        expect(translatorsFrom([row("ada", 5, 0, ["xx"])])).toEqual([]);
    });
});

describe("mergeThanks", () => {
    it("replaces contributors and translators but keeps donators untouched", () => {
        const current = {
            contributors: [{ name: "old" }],
            translators: [{ name: "old", languages: ["de"] }],
            donators: { names: ["Cy"], others: 2 },
        };
        const next = mergeThanks(current, { contributors: [{ name: "ada" }], translators: null });
        expect(next.contributors).toEqual([{ name: "ada" }]);
        expect(next.translators).toEqual(current.translators);
        expect(next.donators).toEqual({ names: ["Cy"], others: 2 });
    });
});
