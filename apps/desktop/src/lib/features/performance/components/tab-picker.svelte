<script lang="ts">
    import { Activity, FileSearch, GitCompare } from "@lucide/svelte";

    import * as Tabs from "$lib/ui/tabs";

    const TABS = [
        {
            value: "frametimes",
            title: "Frametimes",
            blurb: "Record a play session and see your real frame pacing, 1% lows and stutter spikes.",
            icon: Activity,
        },
        {
            value: "scripts",
            title: "Addon scripts",
            blurb: "Check installed addons for script patterns that can hurt performance.",
            icon: FileSearch,
        },
        {
            value: "compare",
            title: "Compare",
            blurb: "Set two saved runs side by side, for example with a mod on and off.",
            icon: GitCompare,
        },
    ];

    let { value = $bindable() }: { value: string } = $props();
</script>

<Tabs.Root bind:value>
    <Tabs.List class="grid h-auto w-full grid-cols-3 gap-3 bg-transparent p-0">
        {#each TABS as t (t.value)}
            <Tabs.Trigger
                value={t.value}
                class="h-auto flex-col items-start gap-1 whitespace-normal rounded-lg border border-border bg-card px-4 py-3 text-left text-foreground hover:border-brass/60 data-[state=active]:border-brass data-[state=active]:bg-brass/10 data-[state=active]:shadow-none"
            >
                <span class="flex items-center gap-2 text-base font-semibold">
                    <t.icon class="size-5 text-brass" />
                    {t.title}
                </span>
                <span class="text-xs font-normal text-muted-foreground">{t.blurb}</span>
            </Tabs.Trigger>
        {/each}
    </Tabs.List>
</Tabs.Root>
