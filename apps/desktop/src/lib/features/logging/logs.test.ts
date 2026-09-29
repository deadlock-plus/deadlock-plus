import { describe, expect, it, vi } from "vitest";
import { filterEntries, forwardConsole, formatArgs, type LogEntry, type LogSink } from "./logs";

const entry = (level: LogEntry["level"], message: string, logger = "tray"): LogEntry => ({
    time: "12:00:00",
    thread: "main",
    level,
    logger,
    message,
});

describe("filterEntries", () => {
    const all = [
        entry("ERROR", "boom"),
        entry("WARN", "hmm"),
        entry("INFO", "ok"),
        entry("DEBUG", "detail", "monitor"),
    ];

    it("keeps the chosen level and everything more severe", () => {
        expect(filterEntries(all, "WARN", "").map((e) => e.level)).toEqual(["ERROR", "WARN"]);
        expect(filterEntries(all, "DEBUG", "")).toHaveLength(4);
    });

    it("searches message and logger, ignoring case", () => {
        expect(filterEntries(all, "DEBUG", "MONITOR")).toHaveLength(1);
        expect(filterEntries(all, "DEBUG", "  BOOM ")).toHaveLength(1);
        expect(filterEntries(all, "DEBUG", "nothing")).toHaveLength(0);
    });
});

describe("formatArgs", () => {
    it("joins strings and stringifies objects", () => {
        expect(formatArgs(["loaded", 3, { a: 1 }])).toBe('loaded 3 {"a":1}');
    });

    it("uses an error's stack, falling back to name and message", () => {
        const withStack = new Error("bad");
        withStack.stack = "Error: bad\n  at x";
        expect(formatArgs([withStack])).toBe("Error: bad\n  at x");
        const bare = new Error("bad");
        bare.stack = undefined;
        expect(formatArgs([bare])).toBe("Error: bad");
    });

    it("survives circular objects", () => {
        const a: Record<string, unknown> = {};
        a.self = a;
        expect(formatArgs([a])).toBe("[object Object]");
    });
});

describe("forwardConsole", () => {
    function setup() {
        const target = { log: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn(), debug: vi.fn() };
        const originals = { ...target };
        const sink: LogSink = { error: vi.fn(), warn: vi.fn(), info: vi.fn(), debug: vi.fn() };
        const restore = forwardConsole(target, sink);
        return { target, originals, sink, restore };
    }

    it("maps each console method to a level and still prints", () => {
        const { target, originals, sink } = setup();
        target.log("a", 1);
        target.info("b");
        target.warn("c");
        target.error("d");
        target.debug("e");
        expect(sink.info).toHaveBeenNthCalledWith(1, "a 1");
        expect(sink.info).toHaveBeenNthCalledWith(2, "b");
        expect(sink.warn).toHaveBeenCalledWith("c");
        expect(sink.error).toHaveBeenCalledWith("d");
        expect(sink.debug).toHaveBeenCalledWith("e");
        expect(originals.log).toHaveBeenCalledWith("a", 1);
        expect(originals.error).toHaveBeenCalledWith("d");
    });

    it("does not loop when the sink itself logs to the console", () => {
        const target = { log: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn(), debug: vi.fn() };
        const sink: LogSink = {
            error: vi.fn(() => target.error("sink failed")),
            warn: vi.fn(),
            info: vi.fn(),
            debug: vi.fn(),
        };
        forwardConsole(target, sink);
        target.error("first");
        expect(sink.error).toHaveBeenCalledTimes(1);
    });

    it("ignores a sink that throws or rejects", async () => {
        const target = { log: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn(), debug: vi.fn() };
        const sink: LogSink = {
            error: vi.fn(() => Promise.reject(new Error("no bridge"))),
            warn: vi.fn(() => {
                throw new Error("sync");
            }),
            info: vi.fn(),
            debug: vi.fn(),
        };
        forwardConsole(target, sink);
        expect(() => target.warn("x")).not.toThrow();
        expect(() => target.error("y")).not.toThrow();
        await Promise.resolve();
    });

    it("restores the original methods", () => {
        const { target, originals, restore } = setup();
        restore();
        expect(target.log).toBe(originals.log);
    });
});
