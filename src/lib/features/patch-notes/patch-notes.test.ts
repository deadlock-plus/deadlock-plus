import { describe, expect, it } from "vitest";
import {
    contentState,
    groupByBullet,
    groupBySection,
    groupBySubject,
    lineParts,
    renderInline,
    type PatchLine,
} from "./patch-notes";

const line = (over: Partial<PatchLine> = {}): PatchLine => ({
    section: "General",
    subject: null,
    tier: null,
    description: "",
    verb: null,
    oldValue: null,
    newValue: null,
    raw: "",
    ...over,
});

describe("groupBySection", () => {
    it("groups consecutive lines under the same section", () => {
        const lines = [
            line({ section: "Heroes", raw: "- Abrams" }),
            line({ section: "Heroes", raw: "- Pocket" }),
            line({ section: "Items", raw: "- Whatever" }),
        ];
        const sections = groupBySection(lines);
        expect(sections).toEqual([
            { name: "Heroes", lines: [lines[0], lines[1]] },
            { name: "Items", lines: [lines[2]] },
        ]);
    });

    it("starts a new group when the same section reappears non-consecutively", () => {
        const lines = [
            line({ section: "Heroes", raw: "a" }),
            line({ section: "Items", raw: "b" }),
            line({ section: "Heroes", raw: "c" }),
        ];
        expect(groupBySection(lines).map((s) => s.name)).toEqual(["Heroes", "Items", "Heroes"]);
    });

    it("returns an empty list for no lines", () => {
        expect(groupBySection([])).toEqual([]);
    });
});

describe("lineParts", () => {
    it("splits a matching subject prefix off the raw text", () => {
        const l = line({ subject: "Abrams", raw: "- Abrams: Infernal Resilience T3 increased from +8% to +9%" });
        expect(lineParts(l)).toEqual({ subject: "Abrams", rest: "Infernal Resilience T3 increased from +8% to +9%" });
    });

    it("falls back to the whole bullet when subject does not actually prefix raw", () => {
        const l = line({ subject: "Abrams", raw: "- Something else entirely" });
        expect(lineParts(l)).toEqual({ subject: null, rest: "Something else entirely" });
    });

    it("falls back to the whole bullet when there is no subject", () => {
        const l = line({ subject: null, raw: "- Guardian bounty increased by 10%" });
        expect(lineParts(l)).toEqual({ subject: null, rest: "Guardian bounty increased by 10%" });
    });
});

describe("groupBySubject", () => {
    it("collapses consecutive lines that share a real subject into one group", () => {
        const lines = [
            line({ subject: "Spiritual Overflow", raw: "- Spiritual Overflow: Buildup is 35% slower" }),
            line({
                subject: "Spiritual Overflow",
                raw: "- Spiritual Overflow: Spirit Power on proc reduced from 40 to 30",
            }),
            line({ subject: "Spiritual Overflow", raw: "- Spiritual Overflow: Fire Rate reduced from 30% to 25%" }),
        ];
        expect(groupBySubject(lines)).toEqual([
            {
                subject: "Spiritual Overflow",
                items: [
                    "Buildup is 35% slower",
                    "Spirit Power on proc reduced from 40 to 30",
                    "Fire Rate reduced from 30% to 25%",
                ],
            },
        ]);
    });

    it("keeps subject-less lines as their own separate groups instead of merging them", () => {
        const lines = [
            line({ subject: null, raw: "- Guardian bounty increased by 10%" }),
            line({ subject: null, raw: "- Walker bounty increased by 5%" }),
        ];
        expect(groupBySubject(lines)).toEqual([
            { subject: null, items: ["Guardian bounty increased by 10%"] },
            { subject: null, items: ["Walker bounty increased by 5%"] },
        ]);
    });

    it("starts a new group when the same subject reappears non-consecutively", () => {
        const lines = [
            line({ subject: "Abrams", raw: "- Abrams: a" }),
            line({ subject: "Pocket", raw: "- Pocket: b" }),
            line({ subject: "Abrams", raw: "- Abrams: c" }),
        ];
        expect(groupBySubject(lines).map((g) => g.subject)).toEqual(["Abrams", "Pocket", "Abrams"]);
    });
});

describe("groupByBullet", () => {
    it("groups consecutive bullet lines together, separate from consecutive prose lines", () => {
        const lines = [
            line({ raw: "- Base HP reduced by 10 for all heroes" }),
            line({ raw: "- HP per boon reduced by 3" }),
            line({ raw: "STANDARD MODE" }),
            line({ raw: "Standard mode provides a lower-stakes way to play." }),
        ];
        const runs = groupByBullet(lines);
        expect(runs).toEqual([
            { bullet: true, lines: [lines[0], lines[1]] },
            { bullet: false, lines: [lines[2], lines[3]] },
        ]);
    });

    it("starts a new run when bullets and prose alternate", () => {
        const lines = [line({ raw: "- a" }), line({ raw: "b" }), line({ raw: "- c" })];
        expect(groupByBullet(lines).map((r) => r.bullet)).toEqual([true, false, true]);
    });

    it("returns an empty list for no lines", () => {
        expect(groupByBullet([])).toEqual([]);
    });
});

describe("renderInline", () => {
    it("renders a bold marker as a <strong> tag", () => {
        expect(renderInline("**STANDARD MODE**")).toBe("<strong>STANDARD MODE</strong>");
    });

    it("renders an italic marker as an <em> tag", () => {
        expect(renderInline("_Each Rank has 6 Subranks_")).toBe("<em>Each Rank has 6 Subranks</em>");
    });

    it("leaves plain text with no markers untouched", () => {
        expect(renderInline("Guardian bounty increased by 10%")).toBe("Guardian bounty increased by 10%");
    });

    it("does not italicize underscores that are just part of a convar/command name", () => {
        expect(renderInline("citadel_enable_slows_affect_air_drag")).toBe("citadel_enable_slows_affect_air_drag");
        expect(renderInline("Fixed a bug with citadel_enable_slows_affect_air_drag")).toBe(
            "Fixed a bug with citadel_enable_slows_affect_air_drag",
        );
    });

    it("still renders a real italic marker sitting next to a convar-like word", () => {
        expect(renderInline("citadel_foo is now _disabled_ by default")).toBe(
            "citadel_foo is now <em>disabled</em> by default",
        );
    });

    it("escapes real HTML characters so source content can never inject markup", () => {
        expect(renderInline("5 < 10 & 10 > 5")).toBe("5 &lt; 10 &amp; 10 &gt; 5");
        expect(renderInline("<img src=x onerror=alert(1)>")).toBe("&lt;img src=x onerror=alert(1)&gt;");
    });
});

describe("contentState", () => {
    it("is empty when there are no lines at all, regardless of origin", () => {
        expect(contentState([], "forum")).toBe("empty");
        expect(contentState([], "steam")).toBe("empty");
    });

    it("is shallow for any forum-origin patch, even one cut mid-sentence with no ellipsis", () => {
        // Live bug: the old marker-based heuristic only caught a preview that happened to carry a
        // literal "...", missing a preview cut mid-sentence with no marker at all.
        const lines = [
            line({ raw: "Deadlock - Six New Heroes - Steam News" }),
            line({ raw: "coming to the streets of the Cursed Apple and features a brand new" }),
        ];
        expect(contentState(lines, "forum")).toBe("shallow");
    });

    it("is full for steam-origin content, even if short", () => {
        expect(contentState([line({ raw: "- One small fix" })], "steam")).toBe("full");
    });

    it("origin decides shallow vs full independent of how many lines there are", () => {
        const lines = Array.from({ length: 10 }, (_, i) => line({ raw: `- Change number ${i}` }));
        expect(contentState(lines, "forum")).toBe("shallow");
        expect(contentState(lines, "steam")).toBe("full");
    });
});
