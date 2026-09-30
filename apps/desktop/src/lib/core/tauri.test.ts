import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
const listenMock = vi.fn();
const getCurrentWindow = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...args: unknown[]) => listenMock(...args) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => getCurrentWindow() }));

import { command, currentWindow, listen } from "./tauri";

describe("tauri wrappers", () => {
    beforeEach(() => {
        invoke.mockReset();
        listenMock.mockReset();
        getCurrentWindow.mockReset();
    });

    it("forwards command name and args to invoke", async () => {
        invoke.mockResolvedValue(3);
        expect(await command<number>("kv_get", { key: "a" })).toBe(3);
        expect(invoke).toHaveBeenCalledWith("kv_get", { key: "a" });
    });

    it("calls commands without args", async () => {
        invoke.mockResolvedValue(undefined);
        await command("ping");
        expect(invoke).toHaveBeenCalledWith("ping", undefined);
    });

    it("rethrows exactly what invoke rejects", async () => {
        const rejection = { kind: "boom" };
        invoke.mockRejectedValue(rejection);
        await expect(command("ping")).rejects.toBe(rejection);
    });

    it("hands the event payload to the handler and resolves to the unlisten fn", async () => {
        const unlisten = vi.fn();
        listenMock.mockImplementation(async (_e: string, cb: (e: { payload: number }) => void) => {
            cb({ payload: 7 });
            return unlisten;
        });
        const handler = vi.fn();
        const stop = await listen<number>("jobs://changed", handler);
        expect(listenMock).toHaveBeenCalledWith("jobs://changed", expect.any(Function));
        expect(handler).toHaveBeenCalledWith(7);
        expect(stop).toBe(unlisten);
    });

    it("returns the current window", () => {
        const win = { minimize: vi.fn() };
        getCurrentWindow.mockReturnValue(win);
        expect(currentWindow()).toBe(win);
    });
});
