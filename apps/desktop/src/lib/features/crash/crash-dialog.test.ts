import { describe, expect, it } from "vitest";
import {
    attachNote,
    crashDetail,
    crashHeading,
    reportFileName,
    shouldShowCrash,
    type CrashReport,
} from "./crash-dialog";

const report = (kind: CrashReport["kind"]): CrashReport => ({
    id: "1",
    kind,
    timestampMs: 0,
    version: "0.5.0",
    os: "Windows",
    message: "boom",
});

describe("wording", () => {
    it("never claims an unclean exit was a crash", () => {
        expect(crashHeading("unclean-exit")).toContain("may have crashed");
        expect(crashDetail("unclean-exit")).not.toMatch(/\bcrashed\b(?! last)/);
    });
    it("says an error happened for panics and web view errors", () => {
        for (const kind of ["panic", "webview"] as const) {
            expect(crashHeading(kind)).toContain("hit an error");
            expect(crashHeading(kind)).not.toContain("may have crashed");
        }
    });
    it("uses the app name", () => {
        expect(crashHeading("panic")).toContain("Deadlock+");
    });
});

describe("shouldShowCrash", () => {
    const base = { report: report("panic"), pathname: "/", dismissed: false, closed: false };
    it("shows for a pending report on a normal page", () => {
        expect(shouldShowCrash(base)).toBe(true);
    });
    it("stays hidden without a report", () => {
        expect(shouldShowCrash({ ...base, report: null })).toBe(false);
    });
    it("waits for onboarding and What's New", () => {
        expect(shouldShowCrash({ ...base, pathname: "/onboarding" })).toBe(false);
        expect(shouldShowCrash({ ...base, pathname: "/onboarding/x" })).toBe(false);
        expect(shouldShowCrash({ ...base, pathname: "/whats-new" })).toBe(false);
    });
    it("does not match look-alike paths", () => {
        expect(shouldShowCrash({ ...base, pathname: "/whats-newer" })).toBe(true);
    });
    it("stays hidden once dismissed or closed", () => {
        expect(shouldShowCrash({ ...base, dismissed: true })).toBe(false);
        expect(shouldShowCrash({ ...base, closed: true })).toBe(false);
    });
});

describe("attach note", () => {
    it("takes the file name from either separator", () => {
        expect(reportFileName(String.raw`C:\Users\x\crashes\report-1.txt`)).toBe("report-1.txt");
        expect(reportFileName("/home/x/crashes/report-1.txt")).toBe("report-1.txt");
    });
    it("names the file to attach and where it is", () => {
        const note = attachNote("/a/report-1.txt");
        expect(note).toContain("report-1.txt");
        expect(note).toContain("/a/report-1.txt");
    });
});
