import { describe, expect, it } from "vitest";
import { featureForPath } from "./track";

describe("featureForPath", () => {
    it("names the home page", () => {
        expect(featureForPath("/")).toBe("home");
    });

    it("uses the first path segment of a feature route", () => {
        expect(featureForPath("/server-picker")).toBe("server-picker");
        expect(featureForPath("/stats/anything/deeper")).toBe("stats");
        expect(featureForPath("/voice-bans/")).toBe("voice-bans");
    });

    it("names the live page", () => {
        expect(featureForPath("/live")).toBe("live");
    });

    it("groups every settings page under one name", () => {
        expect(featureForPath("/settings/privacy")).toBe("settings");
    });

    it("ignores routes that are not features, so nothing free-form leaves the page", () => {
        expect(featureForPath("/onboarding")).toBeNull();
        expect(featureForPath("/whats-new")).toBeNull();
        expect(featureForPath("/users/someone")).toBeNull();
    });
});
