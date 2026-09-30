import { describe, expect, it } from "vitest";
import { closeOutcome } from "./confirm-dialog";

describe("closeOutcome", () => {
    it("does nothing while the dialog opens", () => {
        expect(closeOutcome(true, false)).toBe("none");
    });

    it("cancels a close that did not come from confirm", () => {
        expect(closeOutcome(false, false)).toBe("cancel");
    });

    it("does not cancel a close that followed confirm", () => {
        expect(closeOutcome(false, true)).toBe("none");
    });
});
