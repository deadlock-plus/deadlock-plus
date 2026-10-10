import { beforeEach, describe, expect, it, vi } from "vitest";
import { command } from "$lib/core/tauri";
import { resolveAccoladeInfo } from "./accolade-catalog";

vi.mock("$lib/core/tauri", () => ({ command: vi.fn() }));

const invoke = vi.mocked(command);

describe("resolveAccoladeInfo", () => {
    beforeEach(() => {
        invoke.mockReset();
    });

    it("returns name, description and stat for the ids asked for", async () => {
        invoke.mockResolvedValue([
            { id: 24, name: "The Zapper", description: "{stat_value} ability damage", trackedStat: "ability_damage" },
            { id: 1, name: "Killer Instinct", description: null, trackedStat: "kills" },
        ]);
        const out = await resolveAccoladeInfo([24, 77], "en");
        expect(out.size).toBe(1);
        expect(out.get(24)).toEqual({
            name: "The Zapper",
            description: "{stat_value} ability damage",
            trackedStat: "ability_damage",
        });
        expect(invoke).toHaveBeenCalledWith("game_accolades", { locale: "en" });
    });

    it("is empty without asking the game when there are no ids", async () => {
        expect((await resolveAccoladeInfo([], "en")).size).toBe(0);
        expect(invoke).not.toHaveBeenCalled();
    });

    it("is empty when the game is not available", async () => {
        invoke.mockRejectedValue(new Error("no game"));
        expect((await resolveAccoladeInfo([1], "en")).size).toBe(0);
    });
});
