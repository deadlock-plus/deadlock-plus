import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { waitForPaint } from "./paint";

const host = vi.hoisted(() => ({ platform: "linux" }));
vi.mock("$lib/core/platform", () => host);

let frames: FrameRequestCallback[];

function paintFrame() {
    const pending = frames;
    frames = [];
    for (const callback of pending) callback(0);
}

beforeEach(() => {
    host.platform = "linux";
    frames = [];
    vi.stubGlobal("document", { visibilityState: "hidden" });
    vi.stubGlobal(
        "requestAnimationFrame",
        vi.fn((callback: FrameRequestCallback) => frames.push(callback)),
    );
});

afterEach(() => vi.unstubAllGlobals());

describe("waitForPaint", () => {
    it("lets a hidden Linux window become ready without animation frames", async () => {
        const ready = vi.fn();
        void waitForPaint().then(ready);

        await Promise.resolve();

        expect(ready).toHaveBeenCalledOnce();
        expect(requestAnimationFrame).not.toHaveBeenCalled();
    });

    it.each(["linux", "windows", "macos"])("waits for two frames in a visible %s window", async (platform) => {
        host.platform = platform;
        vi.stubGlobal("document", { visibilityState: "visible" });
        const ready = vi.fn();
        void waitForPaint().then(ready);

        paintFrame();
        await Promise.resolve();
        expect(ready).not.toHaveBeenCalled();

        paintFrame();
        await Promise.resolve();
        expect(ready).toHaveBeenCalledOnce();
    });

    it("keeps the painted-frame wait for a hidden Windows window", async () => {
        host.platform = "windows";
        const ready = vi.fn();
        void waitForPaint().then(ready);

        await Promise.resolve();
        expect(ready).not.toHaveBeenCalled();

        paintFrame();
        await Promise.resolve();
        expect(ready).not.toHaveBeenCalled();

        paintFrame();
        await Promise.resolve();
        expect(ready).toHaveBeenCalledOnce();
    });
});
