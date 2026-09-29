export type Platform = "windows" | "macos" | "linux";

export function detectPlatform(userAgent: string): Platform {
    if (/Macintosh|Mac OS X/i.test(userAgent)) return "macos";
    if (/Linux|X11/i.test(userAgent) && !/Android/i.test(userAgent)) return "linux";
    return "windows";
}

export const platform: Platform = detectPlatform(typeof navigator === "undefined" ? "" : navigator.userAgent);

export function platformName(p: Platform): string {
    return { windows: "Windows", macos: "macOS", linux: "Linux" }[p];
}

export function trashName(p: Platform): string {
    return p === "windows" ? "Recycle Bin" : "Trash";
}
