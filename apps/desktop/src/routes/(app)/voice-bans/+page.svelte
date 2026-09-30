<script lang="ts">
    import { onMount } from "svelte";
    import { createPoller } from "$lib/core/poller";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import ConfirmDialog from "$lib/ui/confirm-dialog.svelte";

    import AddMuteCard from "$lib/features/voice-bans/components/add-mute-card.svelte";
    import FilterBar from "$lib/features/voice-bans/components/filter-bar.svelte";
    import GameRunningBanner from "$lib/features/voice-bans/components/game-running-banner.svelte";
    import HeaderActions from "$lib/features/voice-bans/components/header-actions.svelte";
    import MutedList from "$lib/features/voice-bans/components/muted-list.svelte";
    import Pager from "$lib/features/voice-bans/components/pager.svelte";
    import { plural } from "$lib/features/voice-bans/list";
    import { Mutes } from "$lib/features/voice-bans/mutes.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";

    const POLL_MS = 3000;

    const m = new Mutes();
    const accountLabel = $derived(steamAccount.account?.personaName ?? "your account");

    onMount(() => {
        void m.load();
        const onFocus = () => void m.load();
        window.addEventListener("focus", onFocus);
        const poller = createPoller(
            () => {
                if (!document.hidden) void m.load();
            },
            { intervalMs: POLL_MS },
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
        title="Mutes"
        subtitle="Players muted in Deadlock for {accountLabel}. Every change saves a backup copy of the file first."
    >
        {#snippet actions()}
            <HeaderActions
                importDisabled={m.busy || m.locked || !m.file?.exists}
                exportDisabled={m.muted.length === 0}
                exportLabel="Export {m.selected.size > 0 ? `selected (${m.selected.size})` : 'all'}"
                onimport={(f) => m.importFile(f)}
                onexport={() => m.exportList(m.selected.size > 0 ? [...m.selected] : m.muted)}
            />
        {/snippet}
    </PageHeader>

    {#if m.locked}
        <GameRunningBanner />
    {/if}

    {#if m.error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{m.error}</div>
    {:else if m.loading && !m.file}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Reading voice_ban.dt...</div>
    {:else if m.file && !m.file.exists}
        <EmptyState as="div" layout="fill">
            No voice_ban.dt exists for this account yet. Deadlock creates it the first time you mute someone in game.
        </EmptyState>
    {:else if m.file}
        <AddMuteCard
            bind:value={m.addInput}
            results={m.searchResults}
            mutedSet={m.mutedSet}
            busy={m.busy}
            locked={m.locked}
            searching={m.searching}
            onsubmit={() => m.submitAdd()}
            onmute={(id) => m.mute([id])}
            onopen={(id) => m.openStatlocker(id)}
        />
        <FilterBar
            bind:value={m.filter}
            selectedCount={m.selected.size}
            disabled={m.busy || m.locked}
            onunmute={() => m.askUnmute([...m.selected])}
        />
        <MutedList
            visible={m.visible}
            profiles={m.profiles}
            selected={m.selected}
            allSelected={m.allVisibleSelected}
            filteredCount={m.filtered.length}
            totalCount={m.muted.length}
            filter={m.filter}
            disabled={m.busy || m.locked}
            ontoggleall={(on) => m.toggleVisible(on)}
            ontoggle={(id, on) => m.toggle(id, on)}
            onopen={(id) => m.openStatlocker(id)}
            onunmute={(id) => m.askUnmute([id])}
        />
        {#if m.pageCount > 1}
            <Pager bind:page={m.page} pageCount={m.pageCount} />
        {/if}
    {/if}
</Page>

<ConfirmDialog
    bind:open={m.unmuteOpen}
    title="Unmute {plural(m.unmuteIds.length, 'player')}?"
    description="They are removed from voice_ban.dt. A backup copy of the file is saved beside it first."
    confirmLabel="Unmute"
    onconfirm={() => m.confirmUnmute()}
/>

<ConfirmDialog
    bind:open={m.importOpen}
    title="Import {plural(m.newImportIds.length, 'new mute')}?"
    description="{plural(m.importIds.length, 'id')} in the file, {m.importIds.length -
        m.newImportIds.length} already muted. Existing mutes are kept. A backup copy is saved first."
    confirmLabel="Import"
    disabled={m.newImportIds.length === 0 || m.locked}
    onconfirm={() => m.mute(m.newImportIds)}
/>
