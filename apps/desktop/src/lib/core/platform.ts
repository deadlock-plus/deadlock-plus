import { t } from "./i18n.svelte";

export type Platform = "windows" | "macos" | "linux";

export function detectPlatform(userAgent: string): Platform {
    if (/Macintosh|Mac OS X/i.test(userAgent)) return "macos";
    if (/Linux|X11/i.test(userAgent) && !/Android/i.test(userAgent)) return "linux";
    return "windows";
}

export const platform: Platform = detectPlatform(typeof navigator === "undefined" ? "" : navigator.userAgent);

export function platformName(p: Platform): string {
    if (p === "windows") return t("platform.windows");
    return p === "macos" ? t("platform.macos") : t("platform.linux");
}

export function trashName(p: Platform): string {
    return p === "windows" ? t("platform.recycle_bin") : t("platform.trash");
}
