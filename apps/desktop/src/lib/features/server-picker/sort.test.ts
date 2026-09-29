import { describe, expect, it } from "vitest";
import { DEFAULT_SORT, nextSort, parseSort, sortItems, type SortAccessors } from "./sort";

type Row = { name: string; ping: number | null | undefined; blocked: boolean };

const by: SortAccessors<Row> = { name: (r) => r.name, ping: (r) => r.ping, blocked: (r) => r.blocked };
const names = (rows: Row[]) => rows.map((r) => r.name);

const rows: Row[] = [
    { name: "Frankfurt", ping: 40, blocked: false },
    { name: "Amsterdam", ping: 90, blocked: true },
    { name: "Chicago", ping: null, blocked: false },
    { name: "Berlin", ping: 15, blocked: true },
    { name: "Dubai", ping: undefined, blocked: false },
];

describe("nextSort", () => {
    it("starts a new column ascending", () => {
        expect(nextSort(DEFAULT_SORT, "ping")).toEqual({ key: "ping", dir: "asc" });
    });

    it("flips direction on the active column", () => {
        expect(nextSort({ key: "ping", dir: "asc" }, "ping")).toEqual({ key: "ping", dir: "desc" });
        expect(nextSort({ key: "ping", dir: "desc" }, "ping")).toEqual({ key: "ping", dir: "asc" });
    });
});

describe("sortItems", () => {
    it("defaults to region name A-Z", () => {
        expect(DEFAULT_SORT).toEqual({ key: "region", dir: "asc" });
        expect(names(sortItems(rows, DEFAULT_SORT, by))).toEqual([
            "Amsterdam",
            "Berlin",
            "Chicago",
            "Dubai",
            "Frankfurt",
        ]);
    });

    it("sorts region Z-A", () => {
        expect(names(sortItems(rows, { key: "region", dir: "desc" }, by))[0]).toBe("Frankfurt");
    });

    it("sorts ping lowest first and keeps unanswered rows last", () => {
        expect(names(sortItems(rows, { key: "ping", dir: "asc" }, by))).toEqual([
            "Berlin",
            "Frankfurt",
            "Amsterdam",
            "Chicago",
            "Dubai",
        ]);
    });

    it("sorts ping highest first and still keeps unanswered rows last", () => {
        expect(names(sortItems(rows, { key: "ping", dir: "desc" }, by))).toEqual([
            "Amsterdam",
            "Frankfurt",
            "Berlin",
            "Chicago",
            "Dubai",
        ]);
    });

    it("sorts blocked state, breaking ties by name", () => {
        expect(names(sortItems(rows, { key: "blocked", dir: "asc" }, by))).toEqual([
            "Chicago",
            "Dubai",
            "Frankfurt",
            "Amsterdam",
            "Berlin",
        ]);
        expect(names(sortItems(rows, { key: "blocked", dir: "desc" }, by))).toEqual([
            "Amsterdam",
            "Berlin",
            "Chicago",
            "Dubai",
            "Frankfurt",
        ]);
    });

    it("does not mutate the input", () => {
        const copy = [...rows];
        sortItems(rows, { key: "ping", dir: "asc" }, by);
        expect(rows).toEqual(copy);
    });
});

describe("parseSort", () => {
    it("accepts a valid stored state", () => {
        expect(parseSort({ key: "ping", dir: "desc" })).toEqual({ key: "ping", dir: "desc" });
    });

    it("falls back to the default for anything else", () => {
        for (const bad of [null, undefined, "ping", {}, { key: "nope", dir: "asc" }, { key: "ping", dir: "up" }])
            expect(parseSort(bad)).toEqual(DEFAULT_SORT);
    });
});
