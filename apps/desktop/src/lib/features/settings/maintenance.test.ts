import { describe, expect, it } from "vitest";
import { countdown, formatTime, parseTime } from "./maintenance";

describe("parseTime", () => {
    it("reads HH:MM into minutes of the day", () => {
        expect(parseTime("21:00")).toBe(1260);
        expect(parseTime("00:00")).toBe(0);
        expect(parseTime("23:59")).toBe(1439);
    });

    it("rejects bad input", () => {
        expect(parseTime("")).toBeNull();
        expect(parseTime("24:00")).toBeNull();
        expect(parseTime("12:60")).toBeNull();
        expect(parseTime("nope")).toBeNull();
    });
});

describe("formatTime", () => {
    it("pads to HH:MM and round-trips", () => {
        expect(formatTime(0)).toBe("00:00");
        expect(formatTime(65)).toBe("01:05");
        expect(parseTime(formatTime(1260))).toBe(1260);
    });
});

describe("countdown", () => {
    it("uses the two largest units", () => {
        expect(countdown(0)).toBe("less than a minute");
        expect(countdown(45 * 60)).toBe("45 min");
        expect(countdown(3 * 3600 + 5 * 60)).toBe("3 h 5 min");
        expect(countdown(2 * 86400 + 3 * 3600 + 59 * 60)).toBe("2 d 3 h");
    });

    it("clamps negatives", () => {
        expect(countdown(-30)).toBe("less than a minute");
    });
});
