import type { Platform } from "$lib/core/platform";
import { MAX_PAGE_JUMPS, modifierLabel } from "./shortcuts";

export type CommandGroup = "Pages" | "Settings" | "Actions" | "Keyboard shortcuts";

export interface Command {
    id: string;
    label: string;
    group: CommandGroup;
    keywords: string;
    shortcut?: string;
    run: () => void;
}

export interface CommandDeps {
    pages: { id: string; label: string; href: string }[];
    settingsSections: { id: string; label: string }[];
    platform: Platform;
    go: (href: string) => void;
    checkForUpdates: () => void;
    openSupport: () => void;
}

export function buildCommands(deps: CommandDeps): Command[] {
    const mod = modifierLabel(deps.platform);
    const commands: Command[] = [];

    deps.pages.forEach((page, i) => {
        commands.push({
            id: `page:${page.id}`,
            label: page.label,
            group: "Pages",
            keywords: `go to open ${page.id}`,
            shortcut: i < MAX_PAGE_JUMPS ? `${mod}+${i + 1}` : undefined,
            run: () => deps.go(page.href),
        });
    });

    for (const section of deps.settingsSections) {
        commands.push({
            id: `settings:${section.id}`,
            label: `Settings: ${section.label}`,
            group: "Settings",
            keywords: `preferences options ${section.id}`,
            run: () => deps.go(`/settings/${section.id}`),
        });
    }

    commands.push(
        {
            id: "action:settings",
            label: "Open settings",
            group: "Actions",
            keywords: "preferences options",
            run: () => deps.go("/settings"),
        },
        {
            id: "action:check-updates",
            label: "Check for updates",
            group: "Actions",
            keywords: "upgrade version release",
            run: deps.checkForUpdates,
        },
        {
            id: "action:support",
            label: "Support Deadlock+",
            group: "Actions",
            keywords: "donate ko-fi tip",
            run: deps.openSupport,
        },
        {
            id: "shortcut:palette",
            label: "Open command palette",
            group: "Keyboard shortcuts",
            keywords: "search commands",
            shortcut: `${mod}+K`,
            run: () => {},
        },
        {
            id: "shortcut:jump",
            label: "Jump to a page",
            group: "Keyboard shortcuts",
            keywords: "navigate switch",
            shortcut: `${mod}+1-${MAX_PAGE_JUMPS}`,
            run: () => {},
        },
    );

    return commands;
}
