<script lang="ts">
    import { onMount } from "svelte";
    import { Layers, RefreshCw, Search, ShieldOff } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Input from "$lib/ui/input.svelte";
    import ConfirmDialog from "$lib/ui/confirm-dialog.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import { createPoller } from "$lib/core/poller";

    import { ServerPicker } from "$lib/features/server-picker/picker.svelte";
    import ExternalBlocksBanner from "$lib/features/server-picker/components/external-blocks-banner.svelte";
    import GameRunningBanner from "$lib/features/server-picker/components/game-running-banner.svelte";
    import PresetsDialog from "$lib/features/server-picker/components/presets-dialog.svelte";
    import RegionList from "$lib/features/server-picker/components/region-list.svelte";
    import SortHeader from "$lib/features/server-picker/components/sort-header.svelte";
    import UnsupportedBanner from "$lib/features/server-picker/components/unsupported-banner.svelte";

    const GAME_POLL_MS = 3000;

    const picker = new ServerPicker();

    onMount(() => {
        picker.init();
        void picker.refreshGameRunning();
        const onFocus = () => void picker.refreshGameRunning();
        window.addEventListener("focus", onFocus);
        const poller = createPoller(
            () => {
                if (!document.hidden) void picker.refreshGameRunning();
            },
            { intervalMs: GAME_POLL_MS },
        );
        poller.start();
        return () => {
            window.removeEventListener("focus", onFocus);
            poller.stop();
        };
    });
</script>

<Page>
    <PageHeader
        title="{picker.gameDef?.displayName ?? 'Deadlock'} Server Picker"
        subtitle="Turn a region's toggle on to block it. Blocked regions can't be matched to you, so matchmaking picks from the ones left open."
    >
        {#snippet actions()}
            <Button variant="outline" size="sm" onclick={() => (picker.presetsOpen = true)} disabled={picker.loading}>
                <Layers />
                Presets
            </Button>

            <Button
                variant="outline"
                size="sm"
                onclick={() => picker.pingAll()}
                disabled={picker.loading || picker.pinging}
            >
                <RefreshCw class={picker.pinging ? "animate-spin" : ""} />
                Ping
            </Button>

            <Button
                variant="outline"
                size="sm"
                onclick={() => (picker.unblockAllOpen = true)}
                disabled={picker.blockedIds.size === 0}
            >
                <ShieldOff />
                Unblock all ({picker.blockedIds.size})
            </Button>
        {/snippet}
    </PageHeader>

    {#if picker.gameRunning}
        <GameRunningBanner />
    {/if}

    {#if picker.capability && !picker.capability.supported}
        <UnsupportedBanner />
    {/if}

    {#if picker.externalIds.size > 0}
        <ExternalBlocksBanner
            ruleCount={picker.externalBlocks?.ruleNames.length ?? 0}
            regionCount={picker.externalIds.size}
            label={picker.externalLabel}
            importing={picker.importing}
            onImport={() => (picker.importOpen = true)}
        />
    {/if}

    <div class="relative">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
            bind:value={picker.search}
            placeholder="Filter by region..."
            aria-label="Filter by region"
            class="pl-8"
        />
    </div>

    {#if picker.error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{picker.error}</div>
    {:else if picker.loading && !picker.serverData}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Loading relay data...</div>
    {:else}
        <SortHeader sort={picker.sort} onSort={(key) => picker.sortBy(key)} />
        <RegionList {picker} />
    {/if}
</Page>

<ConfirmDialog
    bind:open={picker.unblockAllOpen}
    title="Unblock all relays?"
    description="This removes all {picker.blockedIds.size} firewall rule{picker.blockedIds.size === 1
        ? ''
        : 's'} Deadlock+ created. Matchmaking will be able to route you to every region again."
    confirmLabel="Unblock all"
    onconfirm={() => picker.unblockAll()}
/>

<ConfirmDialog
    bind:open={picker.importOpen}
    title="Import {picker.externalLabel} blocks?"
    description="Deadlock+ will recreate these blocks as its own rules, then delete the {picker.externalLabel} rules they replace. Any such rule that blocks something outside these regions is left alone."
    confirmLabel="Import"
    onconfirm={() => picker.importExternal()}
/>

<PresetsDialog
    bind:open={picker.presetsOpen}
    presets={picker.presets}
    regions={picker.regions}
    currentlyBlockedRegionIds={picker.blockedRegionIds}
    onApply={(preset) => picker.applyPreset(preset)}
    onSave={(preset) => picker.savePreset(preset)}
    onDelete={(id) => picker.deletePreset(id)}
/>
