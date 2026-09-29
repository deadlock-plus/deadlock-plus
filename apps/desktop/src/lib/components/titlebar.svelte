<script lang="ts">
    import { onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { Copy, Download, Minus, PanelLeftClose, PanelLeftOpen, Square, X } from "@lucide/svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";
    import { updater } from "$lib/features/updates/updater.svelte";
    import NotificationCenter from "$lib/features/notifications/components/notification-center.svelte";

    type Props = {
        sidebarCollapsed: boolean;
        onToggleSidebar: () => void;
    };

    let { sidebarCollapsed, onToggleSidebar }: Props = $props();

    // macOS keeps its native traffic lights (overlay title bar), so it only needs room on the left.
    const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || navigator.userAgent);

    let maximized = $state(false);
    const updateReady = $derived(updater.phase === "available" || updater.phase === "downloading");

    function run(action: (w: ReturnType<typeof getCurrentWindow>) => Promise<unknown>) {
        try {
            void action(getCurrentWindow()).catch(() => {});
        } catch {
            // Not running inside Tauri (plain browser dev): window controls are inert.
        }
    }

    onMount(() => {
        if (isMac) return;
        try {
            const win = getCurrentWindow();
            const sync = () =>
                void win
                    .isMaximized()
                    .then((m) => (maximized = m))
                    .catch(() => {});
            sync();
            const unlisten = win.onResized(sync);
            return () => void unlisten.then((fn) => fn());
        } catch {
            return;
        }
    });
</script>

<header
    data-tauri-drag-region
    class="pointer-events-auto relative z-[60] flex h-10 shrink-0 items-center bg-chrome select-none"
>
    {#if isMac}
        <div data-tauri-drag-region class="h-full w-[78px] shrink-0"></div>
    {/if}

    <button
        type="button"
        onclick={onToggleSidebar}
        aria-label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        class="ml-2 flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground"
    >
        {#if sidebarCollapsed}
            <PanelLeftOpen class="size-4" />
        {:else}
            <PanelLeftClose class="size-4" />
        {/if}
    </button>

    <div
        data-tauri-drag-region
        class="pointer-events-none absolute inset-x-0 top-0 flex h-full items-center justify-center"
    >
        <img src="/favicon.png" alt="" draggable="false" class="mr-2 size-6" />
        <span class="wordmark text-[1.35rem] leading-none">Deadlock<span class="plus">+</span></span>
    </div>

    <div data-tauri-drag-region class="h-full flex-1"></div>

    {#if updateReady}
        <button
            type="button"
            onclick={() => settingsUi.show("about")}
            aria-label={updater.phase === "downloading" ? "Update downloading" : `Update ${updater.version} available`}
            title={updater.phase === "downloading" ? "Downloading update" : `Update ${updater.version} available`}
            class="relative flex h-full w-11.5 items-center justify-center text-primary transition-colors hover:bg-accent/60"
        >
            <Download class="size-4 {updater.phase === 'downloading' ? 'animate-pulse' : ''}" />
            {#if updater.phase === "available"}
                <span class="absolute top-2.5 right-3 size-1.5 rounded-full bg-primary"></span>
            {/if}
        </button>
    {/if}

    <NotificationCenter />

    {#if !isMac}
        <div class="flex h-full">
            <button
                type="button"
                aria-label="Minimize"
                onclick={() => run((w) => w.minimize())}
                class="flex w-11.5 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground"
            >
                <Minus class="size-4" />
            </button>
            <button
                type="button"
                aria-label={maximized ? "Restore" : "Maximize"}
                onclick={() => run((w) => w.toggleMaximize())}
                class="flex w-11.5 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground"
            >
                {#if maximized}
                    <Copy class="size-3.5" />
                {:else}
                    <Square class="size-3.5" />
                {/if}
            </button>
            <button
                type="button"
                aria-label="Close"
                onclick={() => run((w) => w.close())}
                class="flex w-11.5 items-center justify-center text-muted-foreground transition-colors hover:bg-destructive hover:text-destructive-foreground"
            >
                <X class="size-4" />
            </button>
        </div>
    {/if}
</header>

<style>
    .wordmark {
        font-family: "Valve Pulp", var(--font-display);
        font-weight: 700;
        letter-spacing: 0.08em;
        background: linear-gradient(180deg, oklch(0.95 0.05 90), var(--brass) 60%, oklch(0.68 0.11 70));
        -webkit-background-clip: text;
        background-clip: text;
        color: transparent;
        filter: drop-shadow(0 0 10px color-mix(in oklch, var(--brass) 35%, transparent));
    }

    .plus {
        background: none;
        color: var(--primary);
        -webkit-text-fill-color: var(--primary);
    }
</style>
