import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createPoller, type PollerEnv } from "./poller";

function fakeEnv() {
    let hidden = false;
    const listeners = new Set<() => void>();
    const env: PollerEnv = {
        isHidden: () => hidden,
        onVisibilityChange: (cb) => {
            listeners.add(cb);
            return () => listeners.delete(cb);
        },
    };
    return {
        env,
        listeners,
        setHidden(value: boolean) {
            hidden = value;
            for (const cb of [...listeners]) cb();
        },
    };
}

describe("poller", () => {
    beforeEach(() => vi.useFakeTimers());
    afterEach(() => vi.useRealTimers());

    it("does not tick before start", () => {
        const tick = vi.fn();
        createPoller(tick, { intervalMs: 1000 });
        vi.advanceTimersByTime(5000);
        expect(tick).not.toHaveBeenCalled();
    });

    it("ticks every interval after start, not immediately", () => {
        const tick = vi.fn();
        const p = createPoller(tick, { intervalMs: 1000 });
        p.start();
        expect(tick).not.toHaveBeenCalled();
        vi.advanceTimersByTime(3000);
        expect(tick).toHaveBeenCalledTimes(3);
    });

    it("ticks immediately when immediate is set", () => {
        const tick = vi.fn();
        createPoller(tick, { intervalMs: 1000, immediate: true }).start();
        expect(tick).toHaveBeenCalledTimes(1);
        vi.advanceTimersByTime(1000);
        expect(tick).toHaveBeenCalledTimes(2);
    });

    it("stop halts ticking and running reflects state", () => {
        const tick = vi.fn();
        const p = createPoller(tick, { intervalMs: 1000 });
        p.start();
        expect(p.running).toBe(true);
        p.stop();
        expect(p.running).toBe(false);
        vi.advanceTimersByTime(5000);
        expect(tick).not.toHaveBeenCalled();
    });

    it("start twice does not double the rate", () => {
        const tick = vi.fn();
        const p = createPoller(tick, { intervalMs: 1000 });
        p.start();
        p.start();
        vi.advanceTimersByTime(2000);
        expect(tick).toHaveBeenCalledTimes(2);
    });

    it("start returns a stop function", () => {
        const tick = vi.fn();
        const stop = createPoller(tick, { intervalMs: 1000 }).start();
        stop();
        vi.advanceTimersByTime(3000);
        expect(tick).not.toHaveBeenCalled();
    });

    it("can be stopped from inside the tick", () => {
        const tick = vi.fn((p: { stop(): void }) => {
            if (tick.mock.calls.length === 2) p.stop();
        });
        createPoller(tick, { intervalMs: 1000 }).start();
        vi.advanceTimersByTime(10_000);
        expect(tick).toHaveBeenCalledTimes(2);
    });

    it("can restart after stop", () => {
        const tick = vi.fn();
        const p = createPoller(tick, { intervalMs: 1000 });
        p.start();
        p.stop();
        p.start();
        vi.advanceTimersByTime(1000);
        expect(tick).toHaveBeenCalledTimes(1);
    });

    describe("pauseWhenHidden", () => {
        it("pauses while hidden and resumes when visible", () => {
            const { env, setHidden } = fakeEnv();
            const tick = vi.fn();
            const p = createPoller(tick, { intervalMs: 1000, pauseWhenHidden: true, env });
            p.start();
            vi.advanceTimersByTime(1000);
            expect(tick).toHaveBeenCalledTimes(1);
            setHidden(true);
            vi.advanceTimersByTime(5000);
            expect(tick).toHaveBeenCalledTimes(1);
            expect(p.running).toBe(true);
            setHidden(false);
            vi.advanceTimersByTime(1000);
            expect(tick).toHaveBeenCalledTimes(2);
        });

        it("does not schedule when started while hidden", () => {
            const { env, setHidden } = fakeEnv();
            const tick = vi.fn();
            createPoller(tick, { intervalMs: 1000, pauseWhenHidden: true, immediate: true, env }).start();
            setHidden(true);
            vi.advanceTimersByTime(3000);
            expect(tick).toHaveBeenCalledTimes(1);
        });

        it("ignores visibility when not requested", () => {
            const { env, setHidden } = fakeEnv();
            const tick = vi.fn();
            createPoller(tick, { intervalMs: 1000, env }).start();
            setHidden(true);
            vi.advanceTimersByTime(2000);
            expect(tick).toHaveBeenCalledTimes(2);
        });

        it("removes its visibility listener on stop", () => {
            const { env, listeners } = fakeEnv();
            const p = createPoller(vi.fn(), { intervalMs: 1000, pauseWhenHidden: true, env });
            p.start();
            expect(listeners.size).toBe(1);
            p.stop();
            expect(listeners.size).toBe(0);
        });

        it("stays stopped if stopped while hidden then shown", () => {
            const { env, setHidden } = fakeEnv();
            const tick = vi.fn();
            const p = createPoller(tick, { intervalMs: 1000, pauseWhenHidden: true, env });
            p.start();
            setHidden(true);
            p.stop();
            setHidden(false);
            vi.advanceTimersByTime(3000);
            expect(tick).not.toHaveBeenCalled();
        });
    });
});
