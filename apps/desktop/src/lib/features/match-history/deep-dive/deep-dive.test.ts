import { describe, expect, it } from "vitest";
import { parseApiDetail } from "../api-detail";
import { playerBySlot, type MatchDetail, type MatchItem, type MatchPlayer } from "../detail";
import fixture from "../fixtures/api-ranked.json";
import { chartDomain, linePath, playerLine, teamLeadLine, valueAt, type ChartBox } from "./chart";
import { deathRows } from "./deaths";
import { itemBars } from "./items";
import { reduceDamageMatrix } from "./damage";

function detail(): MatchDetail {
    const d = parseApiDetail(fixture);
    if (!d) throw new Error("fixture did not parse");
    return d;
}

const box: ChartBox = { width: 100, height: 50, padLeft: 0, padRight: 0, padTop: 0, padBottom: 0 };

describe("player line", () => {
    it("starts at the origin and follows the series", () => {
        const p = playerBySlot(detail(), 2)!;
        expect(playerLine(p, "souls")).toEqual([
            { t: 0, v: 0 },
            { t: 180, v: 1827 },
            { t: 720, v: 9366 },
            { t: 1570, v: 29528 },
        ]);
        expect(playerLine(p, "playerDamage").at(-1)).toEqual({ t: 1570, v: 22588 });
    });

    it("is empty when the player has no series", () => {
        const p = { ...playerBySlot(detail(), 2)!, series: [] };
        expect(playerLine(p, "souls")).toEqual([]);
    });
});

describe("valueAt", () => {
    const line = [
        { t: 0, v: 0 },
        { t: 100, v: 100 },
        { t: 200, v: 300 },
    ];
    it("interpolates between samples", () => expect(valueAt(line, 150)).toBe(200));
    it("holds the last value after the final sample", () => expect(valueAt(line, 500)).toBe(300));
    it("is 0 for an empty line", () => expect(valueAt([], 5)).toBe(0));
});

describe("chart domain and path", () => {
    it("spans time and value across lines, with a zero floor", () => {
        const dom = chartDomain([
            [
                { t: 0, v: 0 },
                { t: 100, v: 50 },
            ],
            [
                { t: 0, v: 0 },
                { t: 200, v: 80 },
            ],
        ]);
        expect(dom).toEqual({ tMax: 200, vMin: 0, vMax: 80 });
    });

    it("keeps negative values for a lead chart", () => {
        const dom = chartDomain([
            [
                { t: 0, v: 0 },
                { t: 10, v: -40 },
                { t: 20, v: 10 },
            ],
        ]);
        expect(dom.vMin).toBe(-40);
        expect(dom.vMax).toBe(10);
    });

    it("never returns a zero-height domain", () => {
        const dom = chartDomain([[{ t: 0, v: 0 }]]);
        expect(dom.tMax).toBeGreaterThan(0);
        expect(dom.vMax).toBeGreaterThan(dom.vMin);
    });

    it("maps points into the box with y inverted", () => {
        const d = linePath(
            [
                { t: 0, v: 0 },
                { t: 100, v: 100 },
            ],
            { tMax: 100, vMin: 0, vMax: 100 },
            box,
        );
        expect(d).toBe("M0.0 50.0 L100.0 0.0");
    });

    it("returns an empty path for no points", () => {
        expect(linePath([], { tMax: 1, vMin: 0, vMax: 1 }, box)).toBe("");
    });
});

describe("team lead", () => {
    it("is the first team's total minus the second's at each sample time", () => {
        const d = detail();
        const lead = teamLeadLine(d, "souls");
        expect(lead.map((p) => p.t)).toEqual([0, 180, 720, 1570]);
        expect(lead[1].v).toBe(1827 + 1558 - (1934 + 2028));
        expect(lead.at(-1)!.v).toBe(29528 + 22538 - (30399 + 27927));
    });

    it("is empty when no player has a series", () => {
        const d = detail();
        const bare: MatchDetail = {
            ...d,
            teams: d.teams.map((t) => ({
                ...t,
                players: t.players.map((p: MatchPlayer) => ({ ...p, series: [] })),
            })) as unknown as MatchDetail["teams"],
        };
        expect(teamLeadLine(bare, "souls")).toEqual([]);
    });
});

describe("death rows", () => {
    it("lists deaths in time order with the killer's hero when the slot is in the match", () => {
        const d = detail();
        const rows = deathRows(d, playerBySlot(d, 2)!);
        expect(rows.map((r) => r.timeS)).toEqual([261, 442]);
        expect(rows[0]).toMatchObject({ killerSlot: 12, killerHeroId: undefined, respawnS: 8 });
    });

    it("resolves the killer hero from the slot", () => {
        const d = detail();
        const p = playerBySlot(d, 2)!;
        const patched: MatchPlayer = {
            ...p,
            deathLog: [
                { timeS: 500, killerSlot: 8 },
                { timeS: 100, killerSlot: 11 },
            ],
        };
        const rows = deathRows(d, patched);
        expect(rows.map((r) => r.timeS)).toEqual([100, 500]);
        expect(rows[0].killerHeroId).toBe(playerBySlot(d, 11)!.heroId);
        expect(rows[1].killerHeroId).toBe(playerBySlot(d, 8)!.heroId);
    });

    it("names the killer's team, so a row can be coloured by who killed", () => {
        const d = detail();
        const p = playerBySlot(d, 2)!;
        const patched: MatchPlayer = { ...p, deathLog: [{ timeS: 100, killerSlot: 11 }] };
        expect(deathRows(d, patched)[0].killerTeam).toBe(playerBySlot(d, 11)!.team);
        expect(deathRows(d, { ...p, deathLog: [{ timeS: 5, killerSlot: 99 }] })[0].killerTeam).toBeUndefined();
    });
});

describe("item bars", () => {
    const items: MatchItem[] = [
        { itemId: 2, boughtS: 600, upgradeId: 0 },
        { itemId: 1, boughtS: 100, soldS: 400, upgradeId: 0 },
    ];

    it("orders by purchase and ends held items at the match end", () => {
        const bars = itemBars(items, 1000);
        expect(bars.map((b) => b.itemId)).toEqual([1, 2]);
        expect(bars[0]).toMatchObject({ startS: 100, endS: 400, sold: true, left: 10, width: 30 });
        expect(bars[1]).toMatchObject({ startS: 600, endS: 1000, sold: false, left: 60, width: 40 });
    });

    it("keeps a bar visible when sold at the instant of purchase", () => {
        const bars = itemBars([{ itemId: 1, boughtS: 500, soldS: 500, upgradeId: 0 }], 1000);
        expect(bars[0].width).toBeGreaterThan(0);
    });

    it("clamps to the match length and tolerates a zero duration", () => {
        const bars = itemBars([{ itemId: 1, boughtS: 900, upgradeId: 0 }], 0);
        expect(bars[0].left).toBe(0);
        expect(bars[0].width).toBeGreaterThan(0);
    });
});

describe("damage matrix", () => {
    it("totals each dealer to each player, keeping the self slot apart", () => {
        const d = detail();
        const m = reduceDamageMatrix(d);
        expect(m.rows.map((r) => r.slot)).toEqual([2, 4, 8, 11]);
        const row2 = m.rows[0];
        expect(row2.cells.get(8)).toBe(20);
        expect(row2.cells.get(11)).toBe(205);
        expect(row2.cells.has(2)).toBe(false);
        expect(row2.other).toBe(982);
    });

    it("ignores the non-player slot 0", () => {
        const m = reduceDamageMatrix(detail());
        for (const r of m.rows) expect(r.cells.has(0)).toBe(false);
    });

    it("exposes the largest cell for scaling", () => {
        expect(reduceDamageMatrix(detail()).max).toBe(1155);
    });

    it("is empty without entries", () => {
        const d = detail();
        const m = reduceDamageMatrix({ ...d, damageMatrix: { sampleTimesS: [], sources: [], entries: [] } });
        expect(m.rows.every((r) => r.cells.size === 0 && r.other === 0)).toBe(true);
        expect(m.max).toBe(0);
    });
});
