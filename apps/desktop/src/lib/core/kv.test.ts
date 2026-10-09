import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import { kvDelete, kvGet, kvSet } from "./kv";

describe("kv", () => {
    beforeEach(() => {
        invoke.mockReset();
    });

    it("reads a missing key as null", async () => {
        invoke.mockResolvedValue(null);
        expect(await kvGet("app-settings", "theme")).toBeNull();
        expect(invoke).toHaveBeenCalledWith("kv_get", { store: "app-settings", key: "theme" });
    });

    it("returns a stored falsy value as is", async () => {
        invoke.mockResolvedValue(false);
        expect(await kvGet<boolean>("app-settings", "closeToTray")).toBe(false);
    });

    it("passes writes and deletes to the backend", async () => {
        invoke.mockResolvedValue(undefined);
        await kvSet("presets", "presets", [1]);
        await kvDelete("app-settings", "theme");
        expect(invoke).toHaveBeenCalledWith("kv_set", { store: "presets", key: "presets", value: [1] });
        expect(invoke).toHaveBeenCalledWith("kv_delete", { store: "app-settings", key: "theme" });
    });

    it("surfaces backend failures", async () => {
        invoke.mockRejectedValue(new Error("nope"));
        await expect(kvSet("presets", "presets", [])).rejects.toThrow("nope");
    });
});
