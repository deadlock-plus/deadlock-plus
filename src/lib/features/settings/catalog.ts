import type { Component } from "svelte";
import { Bell, Bug, Info, Palette, Power, Scale, Shield } from "@lucide/svelte";

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

export const ITEMS: SettingItem[] = [
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
        id: "accessible-font",
        category: "appearance",
        title: "Accessible font",
        keywords: "atkinson hyperlegible dyslexia low vision readable text typeface",
    },
    {
        id: "autostart",
        category: "startup",
        title: "Start with Windows",
        keywords: "autostart launch boot sign in login startup task scheduler",
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
