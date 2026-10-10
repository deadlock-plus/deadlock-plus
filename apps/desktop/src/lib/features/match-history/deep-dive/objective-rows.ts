import type { MatchObjective, MatchTeam } from "../detail";
import { laneForObjectiveSlot, objectiveLaneSlot, type Lane } from "./lanes";
import { objectiveNames, type ObjectiveKind } from "./names";

export interface ObjectiveRow {
    objectiveId: number;
    kind: ObjectiveKind;
    team: MatchTeam;
    lane?: Lane;
    /** Position among the lanes of this match, only when the lane's colour is unknown. */
    ordinal?: number;
    /** Which of several of the same kind, for shrines. */
    index?: number;
    destroyedS?: number;
    playerDamage: number;
    creepDamage: number;
    spiritDamage: number;
}

export function objectiveRows(objectives: readonly MatchObjective[]): ObjectiveRow[] {
    const names = objectiveNames(objectives);
    return objectives
        .map((o, i): ObjectiveRow => {
            const name = names[i];
            const lane = laneForObjectiveSlot(objectiveLaneSlot(o.objectiveId));
            return {
                objectiveId: o.objectiveId,
                kind: name.kind,
                team: o.team,
                lane,
                ordinal: lane ? undefined : name.lane,
                index: name.index,
                destroyedS: o.destroyedS,
                playerDamage: o.playerDamage,
                creepDamage: o.creepDamage,
                spiritDamage: o.spiritDamage,
            };
        })
        .sort((a, b) => (a.destroyedS ?? Infinity) - (b.destroyedS ?? Infinity));
}

const NAME_KEYS: Record<ObjectiveKind, string> = {
    core: "match_history.deep_dive.objectives.kind_core",
    guardian: "match_history.deep_dive.objectives.kind_guardian",
    walker: "match_history.deep_dive.objectives.kind_walker",
    patron: "match_history.deep_dive.objectives.kind_patron",
    shrine: "match_history.deep_dive.objectives.kind_shrine",
    base_guardian: "match_history.deep_dive.objectives.kind_base_guardian",
    unknown: "match_history.deep_dive.objectives.kind_unknown",
};

export function objectiveNameKey(kind: ObjectiveKind): string {
    return NAME_KEYS[kind];
}
