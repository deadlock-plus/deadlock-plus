import { describe, expect, it } from "vitest";
import { errorReport, errorSummary } from "./report";

describe("errorSummary", () => {
    it("uses the message of an Error", () => {
        expect(errorSummary(new Error("boom"))).toBe("boom");
    });

    it("accepts strings and SvelteKit-style { message } objects", () => {
        expect(errorSummary("plain")).toBe("plain");
        expect(errorSummary({ message: "Not Found" })).toBe("Not Found");
    });

    it("falls back for values with no usable message", () => {
        expect(errorSummary(undefined)).toBe("Unknown error");
        expect(errorSummary({ message: "" })).toBe("Unknown error");
        expect(errorSummary(42)).toBe("Unknown error");
    });
});

describe("errorReport", () => {
    const at = new Date("2026-09-26T12:00:00Z");

    it("includes the message, stack, route and time", () => {
        const err = new Error("boom");
        err.stack = "Error: boom\n    at page.svelte:3:1";
        const text = errorReport(err, { route: "/stats", at });
        expect(text).toContain("Error: boom");
        expect(text).toContain("Route: /stats");
        expect(text).toContain("Time: 2026-09-26T12:00:00.000Z");
        expect(text).toContain("at page.svelte:3:1");
    });

    it("still reports a value that has no stack", () => {
        const text = errorReport("plain", { route: "/", at });
        expect(text).toContain("plain");
        expect(text).not.toContain("undefined");
    });

    it("includes the HTTP status when the route error has one", () => {
        expect(errorReport({ message: "Not Found" }, { route: "/x", at, status: 404 })).toContain("Status: 404");
    });
});
