import { describe, expect, it } from "vitest";
import type { MatchObjective } from "../detail";
import { objectiveRows, objectiveNameKey } from "./objective-rows";

function objective(objectiveId: number, extra: Partial<MatchObjective> = {}): MatchObjective {
    return { objectiveId, team: "hidden-king", creepDamage: 0, playerDamage: 0, spiritDamage: 0, ...extra };
}

describe("objectiveRows", () => {
    it("gives lane objectives their real lane, not their position", () => {
        const rows = objectiveRows([objective(3), objective(4), objective(1)]);
        const lanes = Object.fromEntries(rows.map((r) => [r.objectiveId, r.lane?.color]));
        expect(lanes).toEqual({ 1: "yellow", 3: "blue", 4: "green" });
    });

    it("keeps a numbered fallback when the lane slot has no known colour", () => {
        const [row] = objectiveRows([objective(2)]);
        expect(row.lane).toBeUndefined();
        expect(row.ordinal).toBe(1);
    });

    it("leaves core, patron and shrines without a lane", () => {
        const rows = objectiveRows([objective(0), objective(9), objective(10), objective(11)]);
        expect(rows.map((r) => r.kind)).toEqual(["core", "patron", "shrine", "shrine"]);
        expect(rows.every((r) => r.lane === undefined && r.ordinal === undefined)).toBe(true);
        expect(rows.map((r) => r.index)).toEqual([undefined, undefined, 1, 2]);
    });

    it("sorts by destruction time with standing objectives last", () => {
        const rows = objectiveRows([
            objective(1),
            objective(5, { destroyedS: 600 }),
            objective(6, { destroyedS: 300 }),
        ]);
        expect(rows.map((r) => r.objectiveId)).toEqual([6, 5, 1]);
    });

    it("carries team and damage through", () => {
        const [row] = objectiveRows([
            objective(5, { team: "archmother", destroyedS: 90, playerDamage: 10, creepDamage: 20, spiritDamage: 30 }),
        ]);
        expect(row).toMatchObject({
            team: "archmother",
            destroyedS: 90,
            playerDamage: 10,
            creepDamage: 20,
            spiritDamage: 30,
        });
    });
});

describe("objectiveNameKey", () => {
    it("maps every objective id to a readable kind name", () => {
        const ids = [0, 1, 4, 5, 8, 9, 10, 11, 12, 15, 99];
        const keys = objectiveRows(ids.map((id) => objective(id))).map((r) => [
            r.objectiveId,
            objectiveNameKey(r.kind),
        ]);
        const byId = Object.fromEntries(keys);
        const p = "match_history.deep_dive.objectives.";
        expect(byId).toMatchObject({
            0: `${p}kind_core`,
            1: `${p}kind_guardian`,
            4: `${p}kind_guardian`,
            5: `${p}kind_walker`,
            8: `${p}kind_walker`,
            9: `${p}kind_patron`,
            10: `${p}kind_shrine`,
            11: `${p}kind_shrine`,
            12: `${p}kind_base_guardian`,
            15: `${p}kind_base_guardian`,
            99: `${p}kind_unknown`,
        });
    });
});
