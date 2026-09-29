import { describe, expect, it } from "vitest";
import { autostartLine } from "./autostart";

describe("autostartLine", () => {
    it("is empty when off or still loading", () => {
        expect(autostartLine(null, null)).toBe("");
        expect(autostartLine({ supported: true, enabled: false, stale: false }, null)).toBe("");
    });

    it("says nothing about a healthy setup", () => {
        expect(autostartLine({ supported: true, enabled: true, stale: false }, null)).toBe("");
    });

    it("warns when the task points at another copy", () => {
        expect(autostartLine({ supported: true, enabled: true, stale: true }, null)).toContain(
            "different copy of Deadlock+",
        );
    });

    it("shows the error over any status", () => {
        expect(autostartLine({ supported: true, enabled: true, stale: false }, "Access is denied.")).toBe(
            "Couldn't change it: Access is denied.",
        );
    });
});
