import { describe, expect, it, vi } from "vitest";
import { createOverlays, skipFirst } from "./overlays";

describe("overlays", () => {
    it("closeAll calls every registered close", () => {
        const overlays = createOverlays();
        const a = vi.fn();
        const b = vi.fn();
        overlays.register(a);
        overlays.register(b);
        overlays.closeAll();
        expect(a).toHaveBeenCalledOnce();
        expect(b).toHaveBeenCalledOnce();
    });

    it("an unregistered close is not called", () => {
        const overlays = createOverlays();
        const a = vi.fn();
        const stop = overlays.register(a);
        stop();
        overlays.closeAll();
        expect(a).not.toHaveBeenCalled();
    });

    it("closers that unregister themselves while closing do not skip others", () => {
        const overlays = createOverlays();
        const b = vi.fn();
        const stopA = overlays.register(() => stopA());
        overlays.register(b);
        overlays.closeAll();
        expect(b).toHaveBeenCalledOnce();
        expect(overlays.size).toBe(1);
    });

    it("registering the same close twice keeps two entries until each is removed", () => {
        const overlays = createOverlays();
        const a = vi.fn();
        const stop1 = overlays.register(a);
        overlays.register(a);
        stop1();
        overlays.closeAll();
        expect(a).toHaveBeenCalledOnce();
    });

    it("closeAll is safe to repeat", () => {
        const overlays = createOverlays();
        const a = vi.fn();
        overlays.register(a);
        overlays.closeAll();
        overlays.closeAll();
        expect(a).toHaveBeenCalledTimes(2);
    });
});

describe("skipFirst", () => {
    it("ignores the first call and forwards the rest", () => {
        const fn = vi.fn();
        const wrapped = skipFirst(fn);
        wrapped("a");
        expect(fn).not.toHaveBeenCalled();
        wrapped("b");
        wrapped("c");
        expect(fn.mock.calls).toEqual([["b"], ["c"]]);
    });
});
