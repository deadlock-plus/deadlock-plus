import type { GameBuild } from "$lib/generated/types/GameBuild";
import type { AppInfo } from "$lib/generated/types/AppInfo";
import { platform as currentPlatform, type Platform } from "$lib/core/platform";

export { getAppInfo } from "./api";
export type { GameBuild, AppInfo };

export interface AboutGroup {
    title: string;
    rows: [label: string, value: string][];
}

function present(rows: [string, string | null | undefined][]): [string, string][] {
    return rows.filter((r): r is [string, string] => !!r[1]);
}

const WEBVIEW_LABEL: Record<Platform, string> = { windows: "WebView2", macos: "WKWebView", linux: "WebKitGTK" };

function accountLabel(elevated: boolean, p: Platform): string {
    if (p === "windows") return elevated ? "Administrator" : "Standard user";
    return elevated ? "Root" : "Regular user";
}

export function aboutGroups(info: AppInfo, p: Platform = currentPlatform): AboutGroup[] {
    const build = info.gameBuild;
    const built = [build?.versionDate, build?.versionTime].filter(Boolean).join(" ");
    const groups: AboutGroup[] = [
        {
            title: "Deadlock+",
            rows: present([
                ["Version", info.debugBuild ? `${info.appVersion} (dev build)` : info.appVersion],
                ["Running as", accountLabel(info.elevated, p)],
                ["System", `${info.os} (${info.arch})`],
                ["Tauri", info.tauriVersion],
                [WEBVIEW_LABEL[p], info.webviewVersion],
                ["Data folder", info.dataDir],
            ]),
        },
        {
            title: "Deadlock",
            rows: present([
                ["Game build", build?.clientVersion],
                ["Game built", built],
                ["Source revision", build?.sourceRevision],
                ["Install folder", info.gameDir],
            ]),
        },
    ];
    return groups.filter((g) => g.rows.length > 0);
}

export function aboutText(info: AppInfo, p: Platform = currentPlatform): string {
    return aboutGroups(info, p)
        .map((g) => [g.title, ...g.rows.map(([k, v]) => `${k}: ${v}`)].join("\n"))
        .join("\n\n");
}
