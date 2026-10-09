import { describe, expect, it } from "vitest";
import { reportFileName } from "./report";

describe("reportFileName", () => {
    it("carries the local date so saved reports sort and do not collide across days", () => {
        expect(reportFileName(new Date(2026, 9, 9, 23, 59))).toBe("deadlock-plus-support-2026-10-09.txt");
    });

    it("pads single-digit months and days", () => {
        expect(reportFileName(new Date(2026, 0, 3))).toBe("deadlock-plus-support-2026-01-03.txt");
    });
});
