import { describe, expect, it } from "vitest";
import { returnTarget } from "./return-to";

const nav = (href: string) => ({ url: new URL(href, "http://localhost") });

describe("returnTarget", () => {
    it("remembers the page settings was opened from, with its query", () => {
        expect(returnTarget(nav("/diagnostics?tab=2"))).toBe("/diagnostics?tab=2");
    });

    it("ignores navigation between settings pages", () => {
        expect(returnTarget(nav("/settings/privacy"))).toBeNull();
    });

    it("ignores a missing origin", () => {
        expect(returnTarget(null)).toBeNull();
        expect(returnTarget(undefined)).toBeNull();
    });

    it("ignores an origin without a url, as on the first navigation after a reload", () => {
        expect(returnTarget({ url: null })).toBeNull();
        expect(returnTarget({})).toBeNull();
    });
});
