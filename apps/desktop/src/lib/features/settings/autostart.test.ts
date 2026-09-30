import { describe, expect, it } from "vitest";
import { autostartDescription, autostartLine, autostartTitle } from "./autostart";

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

describe("autostart copy", () => {
    it("names the toggle after the system", () => {
        expect(autostartTitle("windows")).toBe("Start with Windows");
        expect(autostartTitle("macos")).toBe("Start with macOS");
        expect(autostartTitle("linux")).toBe("Start at login");
    });

    it("describes when it launches without naming the wrong system", () => {
        expect(autostartDescription("windows")).toBe("Launches Deadlock+ when you sign in to Windows.");
        expect(autostartDescription("macos")).toBe("Launches Deadlock+ when you log in to macOS.");
        expect(autostartDescription("linux")).toBe("Launches Deadlock+ when you log in to your desktop.");
    });
});
