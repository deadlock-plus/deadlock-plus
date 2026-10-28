import { render } from "svelte/server";
import { describe, expect, it, vi } from "vitest";

vi.mock("$app/state", () => ({ page: { url: new URL("https://deadlockplus.app/download"), status: 200, state: {} } }));

import download from "virtual:en/download";
import { lookup } from "$lib/i18n/core";
import Download from "./+page.svelte";

const release = {
    version: "0.14.0",
    date: "2026-10-20T00:00:00Z",
    windows: "Deadlock+_0.14.0_x64-setup.exe",
    deb: "Deadlock+_0.14.0_amd64.deb",
    appImage: "Deadlock+_0.14.0_amd64.AppImage",
};

const decode = (html: string) =>
    html
        .replace(/<!--.*?-->/g, "")
        .replace(/<[^>]+>/g, "")
        .replaceAll("&amp;", "&")
        .replaceAll("&#39;", "'")
        .replaceAll("&quot;", '"')
        .replace(/\s+/g, " ")
        .trim();

describe("the download page", () => {
    it("tells people stuck on a broken in-app updater to run the installer over their install", () => {
        const { body } = render(Download, { props: { data: { releases: [release] } } as never });
        const note = lookup(download, "download.windows.stuck_update");

        expect(note).toMatch(/0\.13\.0 or 0\.13\.1/);
        expect(body).toMatch(/role="note"/);
        expect(decode(body)).toContain(note);
    });
});
