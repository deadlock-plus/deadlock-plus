<script lang="ts">
    import { page } from "$app/state";
    import AccountChip from "$lib/components/account-chip.svelte";
    import * as Tooltip from "$lib/ui/tooltip";
    import { House, Settings } from "@lucide/svelte";
    import { FEATURES, type FeatureNavEntry } from "$lib/features/registry";
    import { isActivePath } from "$lib/features/home/home";
    import { alerts } from "$lib/features/alerts/alerts.svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";

    const HOME_ENTRY: FeatureNavEntry = {
        id: "home",
        label: "Home",
        href: "/",
        description: "Overview",
        icon: House,
    };

    let { collapsed }: { collapsed: boolean } = $props();
</script>

{#snippet navLink(feature: FeatureNavEntry)}
    {@const active = isActivePath(page.url.pathname, feature.href)}
    {@const Icon = feature.icon}
    <Tooltip.Provider>
        <Tooltip.Root delayDuration={100} disabled={!collapsed}>
            <Tooltip.Trigger>
                {#snippet child({ props })}
                    <a
                        {...props}
                        href={feature.href}
                        aria-label={feature.label}
                        aria-current={active ? "page" : undefined}
                        class="group relative flex h-10 items-center gap-3 overflow-hidden rounded-md px-3.5 font-heading text-sm font-semibold tracking-wide transition-colors {active
                            ? 'bg-accent text-foreground'
                            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
                    >
                        <span
                            class="absolute inset-y-2 left-0 w-0.5 rounded-full bg-brass transition-opacity {active
                                ? 'opacity-100'
                                : 'opacity-0'}"
                        ></span>
                        <Icon class="size-5 shrink-0 {active ? 'text-brass' : ''}" />
                        <span
                            class="truncate whitespace-nowrap transition-opacity duration-200 {collapsed
                                ? 'opacity-0'
                                : 'opacity-100'}">{feature.label}</span
                        >
                        {#if feature.id === "alerts" && alerts.unread > 0}
                            <span
                                class="absolute right-3 top-1/2 size-2 -translate-y-1/2 rounded-full bg-brass"
                                role="status"
                                aria-label="{alerts.unread} unread"
                            ></span>
                        {/if}
                    </a>
                {/snippet}
            </Tooltip.Trigger>
            <Tooltip.Content side="right">{feature.label}</Tooltip.Content>
        </Tooltip.Root>
    </Tooltip.Provider>
{/snippet}

<aside
    class="flex shrink-0 flex-col bg-chrome pb-1 select-none transition-[width] duration-200 {collapsed
        ? 'w-16'
        : 'w-56'}"
>
    <nav class="flex flex-col gap-1 px-2 pt-2">
        {@render navLink(HOME_ENTRY)}
        {#each FEATURES as feature (feature.id)}
            {@render navLink(feature)}
        {/each}
    </nav>

    <div class="mt-auto flex gap-1 px-2 {collapsed ? 'flex-col' : 'items-center'}">
        <div class="min-w-0 {collapsed ? '' : 'flex-1'}">
            <AccountChip {collapsed} />
        </div>
        <Tooltip.Provider>
            <Tooltip.Root delayDuration={100}>
                <Tooltip.Trigger>
                    {#snippet child({ props })}
                        <button
                            {...props}
                            type="button"
                            aria-label="Settings"
                            aria-haspopup="dialog"
                            onclick={() => settingsUi.show()}
                            class="flex h-10 shrink-0 items-center justify-center rounded-md text-muted-foreground {collapsed
                                ? 'w-full'
                                : 'w-10'} transition-colors hover:bg-accent/50 hover:text-foreground"
                        >
                            <Settings class="size-5" />
                        </button>
                    {/snippet}
                </Tooltip.Trigger>
                <Tooltip.Content side="right">Settings</Tooltip.Content>
            </Tooltip.Root>
        </Tooltip.Provider>
    </div>
</aside>
