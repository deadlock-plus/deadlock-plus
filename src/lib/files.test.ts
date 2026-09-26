import { describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import { saveTextFile } from "./files";

const request = { defaultName: "out.json", filterName: "JSON", extension: "json" };

describe("saveTextFile", () => {
    it("sends the file suggestion and contents, never a path", async () => {
        invoke.mockResolvedValueOnce(true);
        await saveTextFile(request, "[]");
        expect(invoke).toHaveBeenCalledWith("save_text_file", { ...request, contents: "[]" });
    });

    it("reports whether the user saved or cancelled", async () => {
        invoke.mockResolvedValueOnce(false);
        expect(await saveTextFile(request, "[]")).toBe(false);
        invoke.mockResolvedValueOnce(true);
        expect(await saveTextFile(request, "[]")).toBe(true);
    });
});
