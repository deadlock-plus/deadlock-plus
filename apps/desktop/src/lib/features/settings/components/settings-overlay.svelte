<script lang="ts">
    import type { Component } from "svelte";
    import { Dialog } from "bits-ui";
    import { Search, X } from "@lucide/svelte";
    import * as Tooltip from "$lib/ui/tooltip";
    import Input from "$lib/ui/input.svelte";
    import { CATEGORIES, matchingCategories, matchingItems, type CategoryId } from "$lib/features/settings/catalog";
    import { settingsUi } from "$lib/features/settings/ui.svelte";
    import Appearance from "./sections/appearance.svelte";
    import Startup from "./sections/startup.svelte";
    import Notifications from "./sections/notifications.svelte";
    import Privacy from "./sections/privacy.svelte";
    import About from "./sections/about.svelte";
    import Diagnostics from "./sections/diagnostics.svelte";
    import Licenses from "./sections/licenses.svelte";

    let { collapsed }: { collapsed: boolean } = $props();

    type Section = Component<{ show: (id: string) => boolean }>;

    const SECTIONS: Record<CategoryId, Section> = {
        appearance: Appearance,
        startup: Startup,
        notifications: Notifications,
        privacy: Privacy,
        about: About,
        diagnostics: Diagnostics,
        licenses: Licenses,
    };

    const hits = $derived(matchingItems(settingsUi.query));
    const visible = $derived(matchingCategories(hits));
    const shown = $derived(hits === null ? [settingsUi.category] : visible);
    const show = (id: string) => hits === null || hits.has(id);
    const logsFill = $derived(settingsUi.logsExpanded && shown.length === 1 && shown[0] === "diagnostics");

    let content = $state<HTMLElement | null>(null);

    function focusSearch(e: Event) {
        e.preventDefault();
        const search = content?.querySelector<HTMLInputElement>('input[type="search"]');
        (search ?? content)?.focus();
    }

    let scroller = $state<HTMLElement | null>(null);

    $effect(() => {
        settingsUi.category;
        settingsUi.query;
        scroller?.scrollTo({ top: 0 });
    });

    function pick(id: CategoryId) {
        settingsUi.category = id;
        settingsUi.query = "";
    }
</script>

{#snippet searchBox()}
    <div class="relative">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
            type="search"
            class="pl-8"
            placeholder="Search settings"
            aria-label="Search settings"
            bind:value={settingsUi.query}
        />
    </div>
{/snippet}

<Dialog.Root bind:open={settingsUi.open}>
    <Dialog.Portal>
        <Dialog.Content
            bind:ref={content}
            interactOutsideBehavior="ignore"
            onOpenAutoFocus={focusSearch}
            onCloseAutoFocus={(e) => e.preventDefault()}
            class="fixed inset-x-0 bottom-7 top-10 z-50 flex bg-chrome outline-none duration-150 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
        >
            <Dialog.Title class="sr-only">Settings</Dialog.Title>
            <Dialog.Description class="sr-only">Search and change Deadlock+ settings.</Dialog.Description>

            <nav
                class="flex shrink-0 flex-col gap-3 px-2 pb-1 pt-2 select-none transition-[width] duration-200 {collapsed
                    ? 'w-16'
                    : 'w-56'}"
                aria-label="Settings categories"
            >
                {#if !collapsed}
                    {@render searchBox()}
                {/if}

                <ul class="flex flex-col gap-1">
                    {#each CATEGORIES as category (category.id)}
                        {#if visible.includes(category.id)}
                            {@const Icon = category.icon}
                            {@const active = hits === null && settingsUi.category === category.id}
                            <li>
                                <Tooltip.Provider>
                                    <Tooltip.Root delayDuration={100} disabled={!collapsed}>
                                        <Tooltip.Trigger>
                                            {#snippet child({ props })}
                                                <button
                                                    {...props}
                                                    type="button"
                                                    aria-current={active ? "page" : undefined}
                                                    aria-label={category.label}
                                                    onclick={() => pick(category.id)}
                                                    class="group relative flex h-10 w-full items-center gap-3 overflow-hidden rounded-md px-3.5 text-left font-heading text-sm font-semibold tracking-wide transition-colors {active
                                                        ? 'bg-accent text-foreground'
                                                        : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
                                                >
                                                    <span
                                                        class="absolute inset-y-2 left-0 w-0.5 rounded-full bg-brass transition-opacity {active
                                                            ? 'opacity-100'
                                                            : 'opacity-0'}"
                                                    ></span>
                                                    <Icon class="size-5 shrink-0 {active ? 'text-brass' : ''}" />
                                                    <span class="truncate whitespace-nowrap">{category.label}</span>
                                                </button>
                                            {/snippet}
                                        </Tooltip.Trigger>
                                        <Tooltip.Content side="right">{category.label}</Tooltip.Content>
                                    </Tooltip.Root>
                                </Tooltip.Provider>
                            </li>
                        {/if}
                    {/each}
                </ul>

                <Dialog.Close
                    aria-label="Close settings"
                    class="mt-auto flex h-10 w-full items-center gap-3 rounded-md px-3.5 text-left font-heading text-sm font-semibold tracking-wide text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                >
                    <X class="size-5 shrink-0" />
                    {#if !collapsed}
                        <span class="flex-1">Close</span>
                        <kbd class="text-xs font-normal text-muted-foreground/70">Esc</kbd>
                    {/if}
                </Dialog.Close>
            </nav>

            <div class="noir-panel relative min-w-0 flex-1 rounded-l-xl border-y border-l border-border">
                {#if logsFill}
                    <div class="h-full p-6">
                        <Diagnostics {show} />
                    </div>
                {:else}
                    <div bind:this={scroller} class="h-full overflow-y-auto">
                        <div class="mx-auto flex max-w-2xl flex-col gap-6 p-6 pb-10">
                            {#if collapsed}
                                {@render searchBox()}
                            {/if}
                            {#if shown.length === 0}
                                <p class="py-16 text-center text-sm text-muted-foreground">
                                    No settings match "{settingsUi.query.trim()}".
                                </p>
                            {/if}
                            {#each shown as id (id)}
                                {@const Body = SECTIONS[id]}
                                {@const label = CATEGORIES.find((c) => c.id === id)?.label}
                                <div class="flex flex-col gap-3">
                                    {#if hits === null}
                                        <h1 class="text-2xl">{label}</h1>
                                    {:else}
                                        <h2
                                            class="font-heading text-xs uppercase tracking-widest text-muted-foreground/70"
                                        >
                                            {label}
                                        </h2>
                                    {/if}
                                    <Body {show} />
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}
            </div>
        </Dialog.Content>
    </Dialog.Portal>
</Dialog.Root>
