import { describe, expect, it } from "vitest";
import { classifyHealth } from "./health";

describe("classifyHealth", () => {
    it("is ok when every service is up", () => {
        expect(classifyHealth({ services: { clickhouse: true, postgres: true, redis: true } })).toEqual({
            level: "ok",
            down: [],
        });
    });

    it("is degraded and names the services that are down", () => {
        expect(classifyHealth({ services: { clickhouse: true, postgres: false, redis: false } })).toEqual({
            level: "degraded",
            down: ["postgres", "redis"],
        });
    });

    it("is down for a malformed body", () => {
        expect(classifyHealth({})).toEqual({ level: "down", down: [] });
        expect(classifyHealth(null)).toEqual({ level: "down", down: [] });
    });
});
