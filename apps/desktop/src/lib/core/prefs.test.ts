import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { PREF_KEYS, prefs } from "./prefs";

function memoryStorage() {
    const data = new Map<string, string>();
    return {
        data,
        getItem: (k: string) => data.get(k) ?? null,
        setItem: (k: string, v: string) => void data.set(k, v),
    };
}

const throwing = {
    getItem: () => {
        throw new Error("denied");
    },
    setItem: () => {
        throw new Error("denied");
    },
};

describe("prefs", () => {
    afterEach(() => {
        vi.unstubAllGlobals();
    });

    describe("with working storage", () => {
        let storage: ReturnType<typeof memoryStorage>;

        beforeEach(() => {
            storage = memoryStorage();
            vi.stubGlobal("localStorage", storage);
        });

        it("keeps the persisted key names", () => {
            expect(PREF_KEYS.sidebarCollapsed).toBe("deadlock-plus:sidebar-collapsed");
            expect(PREF_KEYS.heroCache).toBe("deadlock-plus:heroes");
        });

        it("returns the fallback for a missing key", () => {
            expect(prefs.getString("heroCache", "x")).toBe("x");
            expect(prefs.getString("heroCache")).toBeNull();
            expect(prefs.getBool("sidebarCollapsed", true)).toBe(true);
        });

        it("stores booleans as 1 and 0", () => {
            prefs.setBool("sidebarCollapsed", true);
            expect(storage.data.get("deadlock-plus:sidebar-collapsed")).toBe("1");
            expect(prefs.getBool("sidebarCollapsed", false)).toBe(true);
            prefs.setBool("sidebarCollapsed", false);
            expect(storage.data.get("deadlock-plus:sidebar-collapsed")).toBe("0");
            expect(prefs.getBool("sidebarCollapsed", true)).toBe(false);
        });

        it("round-trips strings", () => {
            prefs.setString("heroCache", '{"at":1}');
            expect(storage.data.get("deadlock-plus:heroes")).toBe('{"at":1}');
            expect(prefs.getString("heroCache")).toBe('{"at":1}');
        });
    });

    describe("with throwing storage", () => {
        beforeEach(() => {
            vi.stubGlobal("localStorage", throwing);
        });

        it("reads fall back", () => {
            expect(prefs.getString("heroCache", "x")).toBe("x");
            expect(prefs.getString("heroCache")).toBeNull();
            expect(prefs.getBool("sidebarCollapsed", true)).toBe(true);
        });

        it("writes do not throw", () => {
            expect(() => prefs.setString("heroCache", "v")).not.toThrow();
            expect(() => prefs.setBool("sidebarCollapsed", true)).not.toThrow();
        });
    });

    describe("with no storage", () => {
        beforeEach(() => {
            vi.stubGlobal("localStorage", undefined);
        });

        it("falls back and ignores writes", () => {
            expect(prefs.getBool("sidebarCollapsed", true)).toBe(true);
            expect(() => prefs.setString("heroCache", "v")).not.toThrow();
        });
    });
});
