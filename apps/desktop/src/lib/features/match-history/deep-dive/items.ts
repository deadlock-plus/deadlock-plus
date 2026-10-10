import type { MatchItem } from "../detail";

export interface ItemBar {
    itemId: number;
    startS: number;
    endS: number;
    sold: boolean;
    imbuedAbilityId?: number;
    /** Percent of the match length. */
    left: number;
    width: number;
}

const MIN_WIDTH = 0.6;

export function itemBars(items: readonly MatchItem[], durationS: number): ItemBar[] {
    return [...items]
        .sort((a, b) => a.boughtS - b.boughtS)
        .map((item) => {
            const sold = item.soldS !== undefined;
            const endS = Math.max(item.soldS ?? durationS, item.boughtS);
            if (durationS <= 0) {
                return {
                    itemId: item.itemId,
                    startS: item.boughtS,
                    endS,
                    sold,
                    imbuedAbilityId: item.imbuedAbilityId,
                    left: 0,
                    width: 100,
                };
            }
            const left = Math.min(Math.max((item.boughtS / durationS) * 100, 0), 100 - MIN_WIDTH);
            const raw = ((endS - item.boughtS) / durationS) * 100;
            const width = Math.min(Math.max(raw, MIN_WIDTH), 100 - left);
            return {
                itemId: item.itemId,
                startS: item.boughtS,
                endS,
                sold,
                imbuedAbilityId: item.imbuedAbilityId,
                left,
                width,
            };
        });
}
