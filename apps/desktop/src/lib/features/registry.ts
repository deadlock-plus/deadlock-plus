import { Activity, Bell, ChartColumn, Film, Globe, Gauge, HardDrive, Timer, TrendingUp, VolumeX } from "@lucide/svelte";
import type { Component } from "svelte";

export interface FeatureNavEntry {
    id: string;
    label: string;
    href: string;
    description: string;
    icon: Component<{ class?: string }>;
}

/**
 * Every top-level feature registers itself here so the app shell's nav stays
 * data-driven. Adding a new feature = a route under src/routes/<id> + an entry here.
 */
export const FEATURES: FeatureNavEntry[] = [
    {
        id: "server-picker",
        label: "Server Picker",
        href: "/server-picker",
        description: "Block or unblock Deadlock's Steam Datagram Relay regions",
        icon: Globe,
    },
    {
        id: "connection",
        label: "Connection",
        href: "/connection",
        description: "Live server, ping and packet loss, with and without ExitLag",
        icon: Activity,
    },
    {
        id: "stats",
        label: "Stats",
        href: "/stats",
        description: "Winrate, streaks and per-hero numbers from your match history",
        icon: ChartColumn,
    },
    {
        id: "rank",
        label: "Rank",
        href: "/rank",
        description: "Your ranked badge over time, calibration and demotion protection",
        icon: TrendingUp,
    },
    {
        id: "sessions",
        label: "Sessions",
        href: "/sessions",
        description: "Your play sessions, and how your results change as they get longer",
        icon: Timer,
    },
    {
        id: "alerts",
        label: "Updates",
        href: "/alerts",
        description: "Recent Deadlock patch notes and Steam announcements",
        icon: Bell,
    },
    {
        id: "performance",
        label: "Performance",
        href: "/performance",
        description: "Scan installed addons' scripts for patterns that can hurt frametimes",
        icon: Gauge,
    },
    {
        id: "voice-bans",
        label: "Mutes",
        href: "/voice-bans",
        description: "View, add, unmute, export and import your muted players",
        icon: VolumeX,
    },
    {
        id: "demos",
        label: "Replays",
        href: "/demos",
        description: "Browse your saved match replays and see which are outdated",
        icon: Film,
    },
    {
        id: "storage",
        label: "Storage",
        href: "/storage",
        description: "See what Deadlock and Deadlock+ use on disk and clear what is safe to",
        icon: HardDrive,
    },
];
