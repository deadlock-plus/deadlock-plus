import { describe, expect, it } from "vitest";
import { needsLaunchNotice, resolveTelemetry, shouldSend } from "./consent";

describe("resolveTelemetry", () => {
    it("defaults to on with the notice not yet shown", () => {
        expect(resolveTelemetry(undefined, undefined)).toEqual({ enabled: true, noticeShown: false });
        expect(resolveTelemetry(null, null)).toEqual({ enabled: true, noticeShown: false });
    });

    it("honours a stored off and a stored shown flag", () => {
        expect(resolveTelemetry(false, true)).toEqual({ enabled: false, noticeShown: true });
    });

    it("treats anything but a stored true as the notice not shown", () => {
        expect(resolveTelemetry(true, "yes" as unknown as boolean)).toEqual({ enabled: true, noticeShown: false });
    });
});

describe("shouldSend", () => {
    it("needs both the switch and a shown notice", () => {
        expect(shouldSend({ enabled: true, noticeShown: true })).toBe(true);
        expect(shouldSend({ enabled: true, noticeShown: false })).toBe(false);
        expect(shouldSend({ enabled: false, noticeShown: true })).toBe(false);
    });
});

describe("needsLaunchNotice", () => {
    it("is wanted once onboarding is behind the user and the notice is unseen", () => {
        expect(needsLaunchNotice({ enabled: true, noticeShown: false }, false)).toBe(true);
    });

    it("leaves the notice to the onboarding step while onboarding is pending", () => {
        expect(needsLaunchNotice({ enabled: true, noticeShown: false }, true)).toBe(false);
    });

    it("never repeats a shown notice", () => {
        expect(needsLaunchNotice({ enabled: true, noticeShown: true }, false)).toBe(false);
    });
});
