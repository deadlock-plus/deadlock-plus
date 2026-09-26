import { describe, expect, it } from "vitest";
import { resolveIngestConsent } from "./ingest-consent";

describe("resolveIngestConsent", () => {
    it("asks and stays off when nothing was stored", () => {
        expect(resolveIngestConsent(undefined)).toEqual({ enabled: false, needsPrompt: true });
        expect(resolveIngestConsent(null)).toEqual({ enabled: false, needsPrompt: true });
    });

    it("honours a stored yes without asking", () => {
        expect(resolveIngestConsent(true)).toEqual({ enabled: true, needsPrompt: false });
    });

    it("honours a stored no without asking", () => {
        expect(resolveIngestConsent(false)).toEqual({ enabled: false, needsPrompt: false });
    });
});
