<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { Command } from "bits-ui";
    import { toast } from "svelte-sonner";
    import { openUrl } from "$lib/core/opener";
    import { platform } from "$lib/core/platform";
    import * as Dialog from "$lib/ui/dialog";
    import { overlayHost } from "$lib/shell/overlay-host.svelte";
    import { NAV_ENTRIES, updater } from "$lib/features/registry";
    import { CATEGORIES } from "$lib/features/settings/catalog";
    import { SUPPORT_URL } from "$lib/features/support/support";
    import { buildCommands, type Command as PaletteCommand, type CommandGroup } from "../commands";
    import { palette } from "../palette.svelte";
    import { isPaletteKey, isTypingTarget, pageJumpIndex } from "../shortcuts";

    const GROUPS: CommandGroup[] = ["Pages", "Settings", "Actions", "Keyboard shortcuts"];

    const commands = buildCommands({
        pages: NAV_ENTRIES,
        settingsSections: CATEGORIES,
        platform,
        go: (href) => void goto(href),
        checkForUpdates: () => {
            void goto("/settings/about");
            void updater.check();
        },
        openSupport: () => void openUrl(SUPPORT_URL).catch((e) => toast.error(`Could not open the link: ${e}`)),
    });

    const enabled = $derived(!page.url.pathname.startsWith("/onboarding"));

    function select(command: PaletteCommand) {
        palette.open = false;
        command.run();
    }

    function onkeydown(e: KeyboardEvent) {
        if (!enabled) return;
        if (isPaletteKey(e, platform)) {
            if (!palette.open && overlayHost.count > 0) return;
            e.preventDefault();
            palette.open = !palette.open;
            return;
        }
        const index = pageJumpIndex(e, platform);
        if (index === null || e.defaultPrevented || overlayHost.count > 0 || isTypingTarget(e.target as HTMLElement)) {
            return;
        }
        const entry = NAV_ENTRIES[index];
        if (!entry) return;
        e.preventDefault();
        void goto(entry.href);
    }
</script>

<svelte:window {onkeydown} />

<Dialog.Root bind:open={palette.open}>
    <Dialog.Content class="max-w-lg gap-0 p-0">
        <Dialog.Title class="sr-only">Command palette</Dialog.Title>
        <Dialog.Description class="sr-only">Search pages, settings and actions.</Dialog.Description>
        <Command.Root loop class="flex min-h-0 flex-col">
            <Command.Input
                placeholder="Search pages, settings and actions"
                aria-label="Search commands"
                class="h-12 w-full border-b border-border bg-transparent px-4 text-sm outline-none placeholder:text-muted-foreground"
            />
            <Command.List class="max-h-80 overflow-y-auto p-2">
                <Command.Viewport>
                    <Command.Empty class="px-3 py-6 text-center text-sm text-muted-foreground">
                        Nothing matches.
                    </Command.Empty>
                    {#each GROUPS as group (group)}
                        <Command.Group>
                            <Command.GroupHeading
                                class="px-3 pb-1 pt-2 font-heading text-xs font-semibold tracking-wide text-muted-foreground"
                            >
                                {group}
                            </Command.GroupHeading>
                            <Command.GroupItems>
                                {#each commands.filter((c) => c.group === group) as command (command.id)}
                                    <Command.Item
                                        value={command.label}
                                        keywords={command.keywords.split(" ")}
                                        onSelect={() => select(command)}
                                        class="flex h-9 cursor-pointer items-center justify-between gap-3 rounded-md px-3 text-sm data-[selected]:bg-accent data-[selected]:text-foreground"
                                    >
                                        <span class="truncate">{command.label}</span>
                                        {#if command.shortcut}
                                            <kbd class="shrink-0 font-sans text-xs text-muted-foreground">
                                                {command.shortcut}
                                            </kbd>
                                        {/if}
                                    </Command.Item>
                                {/each}
                            </Command.GroupItems>
                        </Command.Group>
                    {/each}
                </Command.Viewport>
            </Command.List>
        </Command.Root>
    </Dialog.Content>
</Dialog.Root>
