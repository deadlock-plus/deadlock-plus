import { beforeEach, describe, expect, it, vi } from "vitest";
import { command } from "$lib/core/tauri";
import { loadAbilityArt, loadItemArt } from "./gamedata";

vi.mock("$lib/core/tauri", () => ({
    command: vi.fn(),
    fileSrc: (path: string) => `asset://${path}`,
}));

const invoke = vi.mocked(command);

beforeEach(() => {
    invoke.mockReset();
});

describe("loadItemArt", () => {
    it("maps class names to asset urls", async () => {
        invoke.mockResolvedValue([{ className: "upgrade_a", path: "C:/c/item_upgrade_a_icon.png" }]);
        const out = await loadItemArt(["upgrade_a", "upgrade_b"], "icon");
        expect(invoke).toHaveBeenCalledWith("game_item_art", { names: ["upgrade_a", "upgrade_b"], kind: "icon" });
        expect(out).toEqual({ upgrade_a: "asset://C:/c/item_upgrade_a_icon.png" });
    });

    it("asks for the shop image when told to", async () => {
        invoke.mockResolvedValue([]);
        await loadItemArt(["upgrade_a"], "shop");
        expect(invoke).toHaveBeenCalledWith("game_item_art", { names: ["upgrade_a"], kind: "shop" });
    });

    it("splits large requests into bounded batches and merges them", async () => {
        invoke.mockImplementation(async (_n, args) =>
            (args!.names as string[]).map((className) => ({ className, path: `p/${className}.png` })),
        );
        const names = Array.from({ length: 450 }, (_, i) => `upgrade_${i}`);
        const out = await loadItemArt(names, "icon");
        expect(invoke.mock.calls.map((c) => (c[1]!.names as string[]).length)).toEqual([200, 200, 50]);
        expect(Object.keys(out)).toHaveLength(450);
    });

    it("makes no call for no names", async () => {
        expect(await loadItemArt([], "icon")).toEqual({});
        expect(invoke).not.toHaveBeenCalled();
    });

    it("falls back to nothing when the command fails", async () => {
        invoke.mockRejectedValue(new Error("boom"));
        expect(await loadItemArt(["upgrade_a"], "icon")).toEqual({});
    });
});

describe("loadAbilityArt", () => {
    it("omits abilities the game has no raster icon for", async () => {
        invoke.mockResolvedValue([{ className: "ability_a", path: "p/ability_ability_a.png" }]);
        const out = await loadAbilityArt(["ability_a", "ability_svg"]);
        expect(invoke).toHaveBeenCalledWith("game_ability_art", { names: ["ability_a", "ability_svg"] });
        expect(out).toEqual({ ability_a: "asset://p/ability_ability_a.png" });
        expect(out.ability_svg).toBeUndefined();
    });
});
