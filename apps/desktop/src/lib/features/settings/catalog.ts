import type { Component } from "svelte";
import { Bell, Bug, Info, Palette, Power, Scale, Shield } from "@lucide/svelte";
import { platform, type Platform } from "$lib/core/platform";
import { autostartTitle } from "./autostart";

export type CategoryId = "appearance" | "startup" | "notifications" | "privacy" | "about" | "diagnostics" | "licenses";

export interface Category {
    id: CategoryId;
    label: string;
    icon: Component<{ class?: string }>;
}

export interface SettingItem {
    id: string;
    category: CategoryId;
    title: string;
    keywords: string;
}

export const CATEGORIES: Category[] = [
    { id: "appearance", label: "Appearance", icon: Palette },
    { id: "startup", label: "Startup and background", icon: Power },
    { id: "notifications", label: "Notifications", icon: Bell },
    { id: "privacy", label: "Privacy", icon: Shield },
    { id: "about", label: "Version", icon: Info },
    { id: "diagnostics", label: "Diagnostics", icon: Bug },
    { id: "licenses", label: "Licenses", icon: Scale },
];

const BASE_ITEMS: SettingItem[] = [
    {
        id: "theme",
        category: "appearance",
        title: "Theme",
        keywords: "colour color dark light midnight daylight contrast palette appearance",
    },
    {
        id: "reduced-motion",
        category: "appearance",
        title: "Reduced motion",
        keywords: "animation animations transitions motion accessibility vestibular system",
    },
    {
        id: "language",
        category: "appearance",
        title: "Language",
        keywords: "locale translation english system display",
    },
    {
        id: "accessible-font",
        category: "appearance",
        title: "Accessible font",
        keywords: "atkinson hyperlegible dyslexia low vision readable text typeface",
    },
    {
        id: "autostart",
        category: "startup",
        title: "Start with Windows",
        keywords: "autostart launch boot sign in login log in startup task scheduler launch agent xdg",
    },
    {
        id: "close-to-tray",
        category: "startup",
        title: "Keep running in the tray",
        keywords: "background close hide quit minimize system tray",
    },
    {
        id: "background-jobs",
        category: "startup",
        title: "Background work",
        keywords:
            "pause slow throttle game running cpu performance fps jobs tasks disable off scan addons at launch mods scripts index patch notes automatically updates search embedding model",
    },
    {
        id: "update-alerts",
        category: "notifications",
        title: "Patch and news alerts",
        keywords: "updates patch notes steam announcements toast notify",
    },
    {
        id: "maintenance",
        category: "notifications",
        title: "Steam maintenance reminder",
        keywords: "steam weekly maintenance downtime tuesday wednesday utc remind toast schedule",
    },
    {
        id: "match-ingest",
        category: "privacy",
        title: "Share match data with Deadlock API",
        keywords: "ingest upload replay salts steam id http cache community database telemetry",
    },
    {
        id: "gc-recovery",
        category: "privacy",
        title: "Recover missing match salts through Steam",
        keywords: "gc game coordinator salts steam session token login refresh decrypt ingest matches",
    },
    {
        id: "postgame-capture",
        category: "privacy",
        title: "Instant match results",
        keywords: "postgame post game finished matches stats sessions provisional memory live running game",
    },
    {
        id: "version",
        category: "about",
        title: "Version",
        keywords: "about build app info copy bug report",
    },
    {
        id: "whats-new",
        category: "about",
        title: "What's new",
        keywords: "changelog release notes changes history version",
    },
    {
        id: "updates",
        category: "about",
        title: "App updates",
        keywords: "update upgrade check new version release download install automatic",
    },
    {
        id: "diagnostics",
        category: "diagnostics",
        title: "Diagnostics",
        keywords: "log logs debug session bug report viewer",
    },
    {
        id: "licenses",
        category: "licenses",
        title: "Licenses",
        keywords: "third party notices fonts dependencies open source legal",
    },
];

/** Null means no filter is active. Every word of the query must appear in an item's title or keywords. */
export function itemsFor(p: Platform): SettingItem[] {
    return BASE_ITEMS.filter((item) => item.id !== "postgame-capture" || p === "windows").map((item) =>
        item.id === "autostart" ? { ...item, title: autostartTitle(p) } : item,
    );
}

export const ITEMS: SettingItem[] = itemsFor(platform);

export function matchingItems(query: string): Set<string> | null {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return null;
    const hits = new Set<string>();
    for (const item of ITEMS) {
        const haystack = `${item.title} ${item.keywords}`.toLowerCase();
        if (words.every((w) => haystack.includes(w))) hits.add(item.id);
    }
    return hits;
}

export function matchingCategories(hits: Set<string> | null): CategoryId[] {
    return CATEGORIES.filter((c) => hits === null || ITEMS.some((i) => i.category === c.id && hits.has(i.id))).map(
        (c) => c.id,
    );
}
