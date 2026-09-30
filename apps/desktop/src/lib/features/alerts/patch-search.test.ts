import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { PatchSearch } from "./patch-search.svelte";

function deferred<T>() {
    let resolve!: (v: T) => void;
    const promise = new Promise<T>((r) => (resolve = r));
    return { promise, resolve };
}

describe("PatchSearch", () => {
    beforeEach(() => vi.useFakeTimers());
    afterEach(() => vi.useRealTimers());

    it("waits for the debounce before searching", async () => {
        const run = vi.fn(async (q: string) => [q]);
        const s = new PatchSearch(run, () => {});
        s.query = "hero";
        s.input();
        expect(s.searching).toBe(true);
        await vi.advanceTimersByTimeAsync(249);
        expect(run).not.toHaveBeenCalled();
        await vi.advanceTimersByTimeAsync(1);
        expect(run).toHaveBeenCalledWith("hero");
        expect(s.results).toEqual(["hero"]);
        expect(s.searching).toBe(false);
    });

    it("clears results on an empty query", async () => {
        const run = vi.fn(async () => ["x"]);
        const s = new PatchSearch(run, () => {});
        s.query = "a";
        s.input();
        await vi.advanceTimersByTimeAsync(250);
        s.query = "  ";
        s.input();
        expect(s.results).toEqual([]);
        expect(s.searching).toBe(false);
    });

    it("drops results from a stale search", async () => {
        const first = deferred<string[]>();
        const run = vi.fn((q: string) => (q === "a" ? first.promise : Promise.resolve(["b"])));
        const s = new PatchSearch(run, () => {});
        s.query = "a";
        s.input();
        await vi.advanceTimersByTimeAsync(250);
        s.query = "b";
        s.input();
        await vi.advanceTimersByTimeAsync(250);
        first.resolve(["stale"]);
        await vi.advanceTimersByTimeAsync(0);
        expect(s.results).toEqual(["b"]);
    });

    it("reports a failure only for the latest search", async () => {
        const onError = vi.fn();
        const s = new PatchSearch(() => Promise.reject(new Error("boom")), onError);
        s.query = "a";
        s.input();
        await vi.advanceTimersByTimeAsync(250);
        expect(onError).toHaveBeenCalledOnce();
        expect(s.searching).toBe(false);
    });

    it("clear resets everything and cancels the pending search", async () => {
        const run = vi.fn(async () => ["x"]);
        const s = new PatchSearch(run, () => {});
        s.query = "a";
        s.input();
        s.clear();
        await vi.advanceTimersByTimeAsync(500);
        expect(run).not.toHaveBeenCalled();
        expect(s.query).toBe("");
        expect(s.searching).toBe(false);
    });
});
