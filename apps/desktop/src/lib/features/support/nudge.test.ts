import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const toast = vi.fn();
vi.mock("svelte-sonner", () => ({ toast }));
vi.mock("$lib/core/opener", () => ({ openUrl: vi.fn(async () => {}) }));

function memoryStorage() {
    const data = new Map<string, string>();
    return {
        getItem: (k: string) => data.get(k) ?? null,
        setItem: (k: string, v: string) => void data.set(k, v),
    };
}

describe("support nudge", () => {
    beforeEach(() => {
        vi.resetModules();
        toast.mockClear();
        vi.stubGlobal("localStorage", memoryStorage());
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    async function launch() {
        vi.resetModules();
        const { nudgeOnLaunch } = await import("./nudge");
        nudgeOnLaunch();
    }

    it("stays quiet on the first two launches", async () => {
        await launch();
        await launch();
        expect(toast).not.toHaveBeenCalled();
    });

    it("shows on the third launch only", async () => {
        await launch();
        await launch();
        await launch();
        expect(toast).toHaveBeenCalledTimes(1);
        await launch();
        expect(toast).toHaveBeenCalledTimes(1);
    });

    it("counts a launch once per session", async () => {
        const { nudgeOnLaunch } = await import("./nudge");
        nudgeOnLaunch();
        nudgeOnLaunch();
        nudgeOnLaunch();
        expect(toast).not.toHaveBeenCalled();
    });
});
