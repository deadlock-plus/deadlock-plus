<script lang="ts">
    import type { Component, Snippet } from "svelte";
    import { Package, Zap } from "@lucide/svelte";
    import { t } from "$lib/core/i18n.svelte";
    import Section from "$lib/ui/section.svelte";
    import Unavailable from "./unavailable.svelte";

    type Icon = Component<{ size?: number; class?: string; "aria-hidden"?: boolean | "true" | "false" }>;

    let { title, available, icon, children }: { title: string; available: boolean; icon?: Icon; children: Snippet } =
        $props();

    // Cards that are not handed an icon are matched by their heading.
    const Heading = $derived.by<Icon | undefined>(() => {
        if (icon) return icon;
        if (title === t("match_history.deep_dive.items.heading")) return Package;
        if (title === t("match_history.deep_dive.abilities.heading")) return Zap;
        return undefined;
    });
</script>

<Section>
    <h2 class="mb-3 flex items-center gap-2 text-sm font-medium">
        {#if Heading}
            <Heading size={16} class="shrink-0 text-muted-foreground" aria-hidden="true" />
        {/if}
        {title}
    </h2>
    {#if available}
        {@render children()}
    {:else}
        <Unavailable />
    {/if}
</Section>
