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

describe("saveBinaryFile", () => {
    it("sends the bytes as the raw body and the suggestion as headers, never a path", async () => {
        const { saveBinaryFile } = await import("./files");
        invoke.mockResolvedValueOnce(true);
        const bytes = new Uint8Array([1, 2, 3]);
        await saveBinaryFile({ defaultName: "a.png", extension: "png" }, bytes);
        expect(invoke).toHaveBeenLastCalledWith("save_binary_file", bytes, {
            headers: { "x-default-name": "a.png", "x-extension": "png" },
        });
    });
});
