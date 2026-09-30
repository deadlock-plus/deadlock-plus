<script lang="ts">
    import { goto } from "$app/navigation";
    import { Bell, CalendarClock, Check, Megaphone, Server } from "@lucide/svelte";
    import * as DropdownMenu from "$lib/ui/dropdown-menu";
    import { notifications } from "$lib/features/notifications/notifications.svelte";
    import { formatRelative, type AppNotification } from "$lib/features/notifications/notifications";
    import { settingsUi } from "$lib/features/settings/ui.svelte";

    let open = $state(false);

    function openItem(item: AppNotification) {
        open = false;
        if (item.link) void goto(item.link);
        else if (item.kind === "maintenance") settingsUi.show("notifications");
    }

    function markRead(e: MouseEvent, id: string) {
        e.stopPropagation();
        void notifications.markRead(id);
    }
</script>

<DropdownMenu.Root bind:open>
    <DropdownMenu.Trigger
        aria-label={notifications.unread > 0 ? `${notifications.unread} unread notifications` : "Notifications"}
        title="Notifications"
        class="relative flex h-full w-11.5 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground"
    >
        <Bell class="size-4" />
        {#if notifications.unread > 0}
            <span class="absolute top-2.5 right-3 size-1.5 rounded-full bg-destructive"></span>
        {/if}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-80 p-0" align="end">
        <div class="flex items-center justify-between gap-2 border-b border-border px-3 py-2">
            <span class="font-heading text-sm font-semibold tracking-wide">Notifications</span>
            <button
                type="button"
                disabled={notifications.unread === 0}
                onclick={() => notifications.markAllRead()}
                class="text-xs text-muted-foreground transition-colors hover:text-foreground disabled:opacity-40"
            >
                Mark all read
            </button>
        </div>
        <div class="max-h-96 overflow-y-auto p-1">
            {#if notifications.items.length === 0}
                <p class="px-3 py-6 text-center text-sm text-muted-foreground">No notifications yet.</p>
            {:else}
                {#each notifications.items as item (item.id)}
                    <div class="group flex items-start gap-1 rounded-sm px-1 py-1 transition-colors hover:bg-accent/60">
                        <button
                            type="button"
                            onclick={() => openItem(item)}
                            class="flex min-w-0 flex-1 items-start gap-2.5 rounded-sm px-1.5 py-1 text-left"
                        >
                            {#if item.kind === "maintenance"}
                                <CalendarClock class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                            {:else if item.kind === "servers"}
                                <Server class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                            {:else}
                                <Megaphone class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                            {/if}
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center gap-1.5">
                                    {#if !item.read}
                                        <span class="size-1.5 shrink-0 rounded-full bg-destructive"></span>
                                    {/if}
                                    <span class="truncate text-sm font-medium text-foreground">{item.title}</span>
                                </div>
                                <p class="line-clamp-2 text-xs text-muted-foreground">{item.body}</p>
                                <span class="text-[0.7rem] text-muted-foreground/70"
                                    >{formatRelative(item.timestamp)}</span
                                >
                            </div>
                        </button>
                        {#if !item.read}
                            <button
                                type="button"
                                onclick={(e) => markRead(e, item.id)}
                                aria-label="Mark as read"
                                title="Mark as read"
                                class="mt-1 shrink-0 rounded p-1 text-muted-foreground opacity-0 transition-opacity hover:text-foreground group-hover:opacity-100"
                            >
                                <Check class="size-3.5" />
                            </button>
                        {/if}
                    </div>
                {/each}
            {/if}
        </div>
    </DropdownMenu.Content>
</DropdownMenu.Root>
