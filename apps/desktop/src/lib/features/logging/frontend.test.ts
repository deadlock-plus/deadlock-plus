import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$lib/core/log", () => ({
    debug: vi.fn(() => Promise.resolve()),
    error: vi.fn(() => Promise.resolve()),
    info: vi.fn(() => Promise.resolve()),
    warn: vi.fn(() => Promise.resolve()),
}));
vi.mock("$lib/features/crash/api", () => ({ reportWebviewCrash: vi.fn(() => Promise.resolve()) }));

import { error } from "$lib/core/log";
import { fatalErrorMessage, installFrontendLogging, isFatalError } from "./frontend";

type Listener = (e: unknown) => void;

function fakeWindow() {
    const listeners = new Map<string, Listener>();
    return {
        addEventListener: (type: string, fn: Listener) => void listeners.set(type, fn),
        removeEventListener: (type: string) => void listeners.delete(type),
        fire: (type: string, event: unknown) => listeners.get(type)?.(event),
        has: (type: string) => listeners.has(type),
    };
}

const errorEvent = (over: Record<string, unknown> = {}) => ({
    message: "x is not a function",
    filename: "app.js",
    lineno: 3,
    colno: 9,
    error: new TypeError("x is not a function"),
    ...over,
});

let stop: (() => void) | undefined;
afterEach(() => {
    stop?.();
    stop = undefined;
    vi.clearAllMocks();
});

describe("isFatalError", () => {
    it("treats a thrown error as fatal", () => {
        expect(isFatalError(errorEvent())).toBe(true);
    });

    it("ignores events with no error object", () => {
        expect(isFatalError(errorEvent({ error: null }))).toBe(false);
    });

    it("ignores the benign ResizeObserver notice", () => {
        const message = "ResizeObserver loop completed with undelivered notifications.";
        expect(isFatalError(errorEvent({ message, error: new Error(message) }))).toBe(false);
    });
});

describe("fatalErrorMessage", () => {
    it("carries the message, the location and the stack", () => {
        const text = fatalErrorMessage(errorEvent());
        expect(text).toContain("x is not a function");
        expect(text).toContain("app.js:3:9");
        expect(text).toContain("TypeError");
    });
});

describe("installFrontendLogging", () => {
    it("logs an uncaught error and reports it as a crash", () => {
        const win = fakeWindow();
        const report = vi.fn(() => Promise.resolve());
        stop = installFrontendLogging(win, report);
        win.fire("error", errorEvent());
        expect(error).toHaveBeenCalledOnce();
        expect(report).toHaveBeenCalledOnce();
    });

    it("reports only the first fatal error", () => {
        const win = fakeWindow();
        const report = vi.fn(() => Promise.resolve());
        stop = installFrontendLogging(win, report);
        win.fire("error", errorEvent());
        win.fire("error", errorEvent());
        expect(error).toHaveBeenCalledTimes(2);
        expect(report).toHaveBeenCalledOnce();
    });

    it("logs an unhandled rejection without reporting a crash", () => {
        const win = fakeWindow();
        const report = vi.fn(() => Promise.resolve());
        stop = installFrontendLogging(win, report);
        win.fire("unhandledrejection", { reason: new Error("network down") });
        expect(error).toHaveBeenCalledOnce();
        expect(report).not.toHaveBeenCalled();
    });

    it("does not report a non-fatal error event", () => {
        const win = fakeWindow();
        const report = vi.fn(() => Promise.resolve());
        stop = installFrontendLogging(win, report);
        win.fire("error", errorEvent({ error: null }));
        expect(report).not.toHaveBeenCalled();
    });

    it("survives a failing report", () => {
        const win = fakeWindow();
        stop = installFrontendLogging(win, () => Promise.reject(new Error("ipc down")));
        expect(() => win.fire("error", errorEvent())).not.toThrow();
    });

    it("removes its listeners when stopped", () => {
        const win = fakeWindow();
        installFrontendLogging(win, vi.fn())();
        expect(win.has("error")).toBe(false);
        expect(win.has("unhandledrejection")).toBe(false);
    });
});
