import { Activity, Bell, ChartColumn, Film, Globe, Gauge, HardDrive, Timer, TrendingUp, VolumeX } from "@lucide/svelte";
import type { Component } from "svelte";
import { installFrontendLogging } from "./logging/frontend";
import { apiHealth } from "./api-health/health.svelte";
import { alerts } from "./alerts/alerts.svelte";
import { connectivity } from "./connectivity/online.svelte";
import { gcStatus } from "./gc/status.svelte";
import { ingestStatus } from "./ingest/status.svelte";
import { jobs } from "./jobs/jobs.svelte";
import { notifications } from "./notifications/notifications.svelte";
import { onboarding } from "./onboarding/onboarding.svelte";
import { performanceScan } from "./performance/scan.svelte";
import { settings } from "./settings/settings.svelte";
import { steamAccount } from "./steam-account/account.svelte";
import { checkOnLaunch, startBackgroundUpdateChecks } from "./updates/launch-check";
import { whatsNew } from "./updates/whats-new.svelte";

export interface FeatureNavEntry {
    id: string;
    label: string;
    href: string;
    description: string;
    icon: Component<{ class?: string }>;
    /** Count shown as a dot on the nav link while above zero. */
    badge?: () => number;
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
        description: "Live server, ping and packet loss",
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
        badge: () => alerts.unread,
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

/** Started in order when the app mounts; a returned function stops the service. */
export type Service = () => void | (() => void);

export const SERVICES: Service[] = [
    installFrontendLogging,
    () => {
        void settings.init();
    },
    () => ingestStatus.start(),
    () => gcStatus.start(),
    () => apiHealth.start(),
    () => steamAccount.start(),
    () => alerts.start(),
    () => notifications.start(),
    () => jobs.start(),
    () => performanceScan.start(),
    () => connectivity.start(),
    () => {
        void onboarding.init();
    },
    () => {
        void whatsNew.init();
    },
    () => void checkOnLaunch(),
    startBackgroundUpdateChecks,
];

export { isActivePath } from "./home/home";
export { apiHealth, ingestStatus, jobs, performanceScan, settings };
export { jobPercent, jobStatusText } from "./jobs/jobs";
export { isLightTheme, resolveReducedMotion } from "./settings/themes";
export { updater } from "./updates/updater.svelte";
export { default as NotificationCenter } from "./notifications/components/notification-center.svelte";
