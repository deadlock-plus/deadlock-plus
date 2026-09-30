import { describe, expect, it, vi } from "vitest";
import { deeplinkFor, openWithDeeplink } from "./deeplink";

describe("deeplinkFor", () => {
    it.each([
        "https://store.steampowered.com/news/app/1422450/view/123",
        "https://steamcommunity.com/games/1422450/announcements/detail/456",
        "https://store.steampowered.com/account/playtestinvites",
        "https://store.steampowered.com/",
        "https://help.steampowered.com/en/",
        "https://steamcommunity.com/id/someone/games",
    ])("maps %s to steam://openurl", (url) => {
        expect(deeplinkFor(url)).toBe(`steam://openurl/${url}`);
    });

    it("maps store app pages to the native store page", () => {
        expect(deeplinkFor("https://store.steampowered.com/app/1422450/Deadlock/")).toBe("steam://store/1422450");
        expect(deeplinkFor("https://store.steampowered.com/app/1422450")).toBe("steam://store/1422450");
    });

    it("passes steam:// links through unchanged", () => {
        expect(deeplinkFor("steam://nav/games/details/1422450")).toBe("steam://nav/games/details/1422450");
    });

    it("ignores other hosts and lookalikes", () => {
        expect(deeplinkFor("https://forums.playdeadlock.com/threads/1")).toBeNull();
        expect(deeplinkFor("https://store.steampowered.com.evil.test/x")).toBeNull();
        expect(deeplinkFor("https://evil.test/?u=store.steampowered.com")).toBeNull();
    });

    it("ignores unparseable input", () => {
        expect(deeplinkFor("nope")).toBeNull();
    });
});

describe("openWithDeeplink", () => {
    const url = "https://store.steampowered.com/news/app/1422450/view/123";

    it("opens the deeplink when it works", async () => {
        const open = vi.fn().mockResolvedValue(undefined);
        await openWithDeeplink(url, open);
        expect(open).toHaveBeenCalledTimes(1);
        expect(open).toHaveBeenCalledWith(`steam://openurl/${url}`);
    });

    it("falls back to the browser when the deeplink fails", async () => {
        const open = vi.fn().mockRejectedValueOnce(new Error("no handler")).mockResolvedValueOnce(undefined);
        await openWithDeeplink(url, open);
        expect(open).toHaveBeenNthCalledWith(2, url);
    });

    it("does not retry a steam:// link in the browser", async () => {
        const open = vi.fn().mockRejectedValue(new Error("no handler"));
        await expect(openWithDeeplink("steam://nav/games", open)).rejects.toThrow("no handler");
        expect(open).toHaveBeenCalledTimes(1);
    });

    it("opens the plain URL when there is no deeplink", async () => {
        const open = vi.fn().mockResolvedValue(undefined);
        await openWithDeeplink("https://example.test/x", open);
        expect(open).toHaveBeenCalledExactlyOnceWith("https://example.test/x");
    });

    it("surfaces a browser failure", async () => {
        const open = vi.fn().mockRejectedValue(new Error("boom"));
        await expect(openWithDeeplink("https://example.test/x", open)).rejects.toThrow("boom");
    });
});
