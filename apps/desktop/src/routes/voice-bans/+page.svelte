<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { saveTextFile } from "$lib/files";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import {
        ChevronLeft,
        ChevronRight,
        Download,
        ExternalLink,
        Plus,
        Search,
        TriangleAlert,
        Upload,
        Volume2,
    } from "@lucide/svelte";

    import Button from "$lib/components/ui/button.svelte";
    import Input from "$lib/components/ui/input.svelte";
    import Badge from "$lib/components/ui/badge.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";

    import { readVoiceBan, writeVoiceBan, type VoiceBanFile } from "$lib/features/voice-bans/api";
    import { lookupProfiles, searchPlayers, type Profile } from "$lib/features/voice-bans/profiles";
    import {
        addMutedUsers,
        buildExport,
        parseImport,
        parseSteamId,
        parseVoiceBan,
        removeMutedUsers,
        statlockerProfileUrl,
        steam64ToSteam32,
    } from "$lib/features/voice-bans/voice-ban";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";

    const PAGE_SIZE = 25;
    const POLL_MS = 3000;

    let file = $state<VoiceBanFile | null>(null);
    let error = $state<string | null>(null);
    let loading = $state(true);
    let busy = $state(false);
    let filter = $state("");
    let page = $state(0);
    let selected = $state<Set<string>>(new Set());
    let profiles = $state<Record<string, Profile>>({});
    const requested = new Set<string>();
    let addInput = $state("");
    let searchResults = $state<Profile[] | null>(null);
    let searching = $state(false);
    let unmuteOpen = $state(false);
    let unmuteIds = $state<string[]>([]);
    let importOpen = $state(false);
    let importIds = $state<string[]>([]);
    let fileInput: HTMLInputElement;

    // Newest first: the game appends new mutes to the end of the file.
    const muted = $derived.by(() => {
        if (!file?.exists) return [];
        try {
            return parseVoiceBan(file.text)
                .users.map((u) => u.steamid64)
                .reverse();
        } catch {
            return [];
        }
    });
    const mutedSet = $derived(new Set(muted));

    const filtered = $derived.by(() => {
        const q = filter.trim().toLowerCase();
        if (!q) return muted;
        return muted.filter(
            (id) => id.includes(q) || steam64ToSteam32(id).includes(q) || profiles[id]?.name.toLowerCase().includes(q),
        );
    });
    const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / PAGE_SIZE)));
    const visible = $derived(filtered.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE));
    const allVisibleSelected = $derived(visible.length > 0 && visible.every((id) => selected.has(id)));
    const newImportIds = $derived(importIds.filter((id) => !mutedSet.has(id)));
    const locked = $derived(file?.gameRunning === true);
    const accountLabel = $derived(steamAccount.account?.personaName ?? "your account");

    async function load() {
        loading = true;
        try {
            file = await readVoiceBan();
            error = null;
            const known = new Set(file.exists ? parseVoiceBan(file.text).users.map((u) => u.steamid64) : []);
            selected = new Set([...selected].filter((id) => known.has(id)));
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    function fetchProfiles(ids: string[]) {
        const fresh = ids.filter((id) => !requested.has(id));
        fresh.forEach((id) => requested.add(id));
        if (fresh.length === 0) return;
        void lookupProfiles(fresh, (batch) => {
            const next = { ...profiles };
            for (const p of batch) next[p.steamid64] = p;
            profiles = next;
        });
    }

    $effect(() => {
        fetchProfiles(visible);
    });

    // Name filtering needs every profile, so the rest load once the user starts searching.
    $effect(() => {
        if (filter.trim()) fetchProfiles(muted);
    });

    $effect(() => {
        filter;
        page = 0;
    });

    $effect(() => {
        if (page >= pageCount) page = pageCount - 1;
    });

    onMount(() => {
        void load();
        const onFocus = () => void load();
        window.addEventListener("focus", onFocus);
        const timer = setInterval(() => {
            if (!document.hidden) void load();
        }, POLL_MS);
        return () => {
            window.removeEventListener("focus", onFocus);
            clearInterval(timer);
        };
    });

    async function commit(text: string, message: string) {
        if (locked) return;
        busy = true;
        try {
            const backup = await writeVoiceBan(text);
            toast.success(message, { description: `Backup: ${backup}` });
            await load();
        } catch (e) {
            toast.error(String(e));
        } finally {
            busy = false;
        }
    }

    function askUnmute(ids: string[]) {
        unmuteIds = ids;
        unmuteOpen = true;
    }

    async function confirmUnmute() {
        if (!file) return;
        const count = unmuteIds.length;
        await commit(removeMutedUsers(file.text, unmuteIds), `Unmuted ${count} player${count === 1 ? "" : "s"}`);
        selected = new Set();
    }

    async function mute(ids: string[]) {
        if (!file) return;
        const { text, added } = addMutedUsers(file.text, ids);
        if (added.length === 0) {
            toast.info("Already muted.");
            return;
        }
        await commit(text, `Muted ${added.length} player${added.length === 1 ? "" : "s"}`);
        addInput = "";
        searchResults = null;
    }

    async function submitAdd() {
        const value = addInput.trim();
        if (!value) return;
        const id = parseSteamId(value);
        if (id) {
            await mute([id]);
            return;
        }
        if (/^https?:\/\//i.test(value) || /^\d+$/.test(value)) {
            toast.error("Not recognised. Use a SteamID64, account ID or /profiles/ link.");
            return;
        }
        searching = true;
        try {
            searchResults = await searchPlayers(value);
        } catch (e) {
            toast.error(`Search failed: ${e}`);
        } finally {
            searching = false;
        }
    }

    function openStatlocker(id: string) {
        openUrl(statlockerProfileUrl(id)).catch((e) => toast.error(`Could not open Statlocker: ${e}`));
    }

    function toggle(id: string, on: boolean) {
        const next = new Set(selected);
        if (on) next.add(id);
        else next.delete(id);
        selected = next;
    }

    function toggleVisible(on: boolean) {
        const next = new Set(selected);
        for (const id of visible) {
            if (on) next.add(id);
            else next.delete(id);
        }
        selected = next;
    }

    async function exportList(ids: string[]) {
        try {
            const saved = await saveTextFile(
                { defaultName: "deadlock-mutes.json", filterName: "JSON", extension: "json" },
                buildExport(ids),
            );
            if (saved) toast.success(`Exported ${ids.length} mutes`);
        } catch (e) {
            toast.error(`Export failed: ${e instanceof Error ? e.message : e}`);
        }
    }

    async function onImportFile(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const chosen = input.files?.[0];
        input.value = "";
        if (!chosen) return;
        try {
            importIds = parseImport(await chosen.text());
            importOpen = true;
        } catch (e) {
            toast.error(`Import failed: ${e instanceof Error ? e.message : e}`);
        }
    }
</script>

<div class="mx-auto flex min-h-full max-w-4xl flex-col gap-4 px-6 pb-10 pt-6">
    <header class="flex items-start justify-between gap-4">
        <div>
            <h1 class="text-2xl">Mutes</h1>
            <p class="text-sm text-muted-foreground">
                Players muted in Deadlock for {accountLabel}. Every change saves a backup copy of the file first.
            </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <Button
                variant="outline"
                size="sm"
                onclick={() => fileInput.click()}
                disabled={busy || locked || !file?.exists}
            >
                <Upload />
                Import
            </Button>
            <Button
                variant="outline"
                size="sm"
                onclick={() => exportList(selected.size > 0 ? [...selected] : muted)}
                disabled={muted.length === 0}
            >
                <Download />
                Export {selected.size > 0 ? `selected (${selected.size})` : "all"}
            </Button>
            <input
                bind:this={fileInput}
                type="file"
                accept=".json,.dt,text/plain,application/json"
                class="hidden"
                onchange={onImportFile}
            />
        </div>
    </header>

    {#if locked}
        <div
            class="flex items-center gap-4 rounded-lg border-2 border-destructive/60 bg-destructive/10 px-5 py-4 text-destructive"
        >
            <TriangleAlert class="size-8 shrink-0" />
            <div>
                <p class="font-heading text-lg font-semibold">Deadlock is running</p>
                <p class="text-sm">
                    Mutes can't be changed while the game is open. It rewrites the file when it closes and would erase
                    your changes. Close Deadlock to edit this list.
                </p>
            </div>
        </div>
    {/if}

    {#if error}
        <div class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</div>
    {:else if loading && !file}
        <div class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Reading voice_ban.dt...</div>
    {:else if file && !file.exists}
        <div class="flex flex-1 items-center justify-center px-6 text-center text-sm text-muted-foreground">
            No voice_ban.dt exists for this account yet. Deadlock creates it the first time you mute someone in game.
        </div>
    {:else if file}
        <div class="flex flex-col gap-2 rounded-md border border-border bg-card p-3">
            <div class="flex gap-2">
                <Input
                    bind:value={addInput}
                    placeholder="Mute by SteamID64, account ID, /profiles/ link or player name"
                    aria-label="Player to mute"
                    onkeydown={(e) => e.key === "Enter" && submitAdd()}
                />
                <Button onclick={submitAdd} disabled={busy || locked || searching || !addInput.trim()}>
                    <Plus />
                    {parseSteamId(addInput) ? "Mute" : "Search"}
                </Button>
            </div>
            {#if searchResults}
                {#if searchResults.length === 0}
                    <p class="text-sm text-muted-foreground">No players found for that name.</p>
                {:else}
                    <ul class="flex flex-col gap-1">
                        {#each searchResults as p (p.steamid64)}
                            <li class="flex items-center gap-3 rounded-md px-2 py-1.5 hover:bg-accent/40">
                                {#if p.avatar}<img src={p.avatar} alt="" class="size-8 rounded-sm" />{/if}
                                <div class="min-w-0 flex-1">
                                    <p class="truncate text-sm font-medium">{p.name}</p>
                                    <p class="text-xs text-muted-foreground">{p.steamid64}</p>
                                </div>
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    aria-label="Open Statlocker profile"
                                    title="Statlocker profile"
                                    onclick={() => openStatlocker(p.steamid64)}
                                >
                                    <ExternalLink />
                                </Button>
                                {#if mutedSet.has(p.steamid64)}
                                    <Badge variant="secondary">Muted</Badge>
                                {:else}
                                    <Button
                                        size="sm"
                                        variant="outline"
                                        disabled={busy || locked}
                                        onclick={() => mute([p.steamid64])}>Mute</Button
                                    >
                                {/if}
                            </li>
                        {/each}
                    </ul>
                {/if}
            {/if}
        </div>

        <div class="flex items-center gap-2">
            <div class="relative flex-1">
                <Search
                    class="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                />
                <Input
                    bind:value={filter}
                    placeholder="Filter by name or ID..."
                    aria-label="Filter by name or ID"
                    class="pl-8"
                />
            </div>
            <Button
                variant="outline"
                size="sm"
                disabled={selected.size === 0 || busy || locked}
                onclick={() => askUnmute([...selected])}
            >
                <Volume2 />
                Unmute selected ({selected.size})
            </Button>
        </div>

        <div class="flex items-center gap-3 px-4 text-xs font-medium text-muted-foreground">
            <input
                type="checkbox"
                aria-label="Select this page"
                checked={allVisibleSelected}
                onchange={(e) => toggleVisible(e.currentTarget.checked)}
            />
            <span class="flex-1">{filtered.length} of {muted.length} muted</span>
        </div>

        <ul class="flex flex-col gap-1.5">
            {#each visible as id (id)}
                {@const p = profiles[id]}
                <li class="flex items-center gap-3 rounded-md border border-border bg-card px-4 py-2">
                    <input
                        type="checkbox"
                        aria-label="Select {p?.name ?? id}"
                        checked={selected.has(id)}
                        onchange={(e) => toggle(id, e.currentTarget.checked)}
                    />
                    {#if p?.avatar}
                        <img src={p.avatar} alt="" class="size-8 shrink-0 rounded-sm" />
                    {:else}
                        <div class="size-8 shrink-0 rounded-sm bg-accent"></div>
                    {/if}
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-medium">{p?.name ?? "Unknown player"}</p>
                        <p class="text-xs text-muted-foreground">{id}</p>
                    </div>
                    <Button
                        size="sm"
                        variant="ghost"
                        aria-label="Open Statlocker profile"
                        title="Statlocker profile"
                        onclick={() => openStatlocker(id)}
                    >
                        <ExternalLink />
                    </Button>
                    <Button size="sm" variant="outline" disabled={busy || locked} onclick={() => askUnmute([id])}
                        >Unmute</Button
                    >
                </li>
            {:else}
                <li class="py-8 text-center text-sm text-muted-foreground">
                    {muted.length === 0 ? "Nobody is muted." : `No one matches "${filter}".`}
                </li>
            {/each}
        </ul>

        {#if pageCount > 1}
            <div class="flex items-center justify-center gap-3">
                <Button
                    variant="outline"
                    size="icon"
                    aria-label="Previous page"
                    disabled={page === 0}
                    onclick={() => page--}
                >
                    <ChevronLeft />
                </Button>
                <span class="text-sm text-muted-foreground">Page {page + 1} of {pageCount}</span>
                <Button
                    variant="outline"
                    size="icon"
                    aria-label="Next page"
                    disabled={page >= pageCount - 1}
                    onclick={() => page++}
                >
                    <ChevronRight />
                </Button>
            </div>
        {/if}
    {/if}
</div>

<AlertDialog.Root bind:open={unmuteOpen}>
    <AlertDialog.Content>
        <AlertDialog.Title>Unmute {unmuteIds.length} player{unmuteIds.length === 1 ? "" : "s"}?</AlertDialog.Title>
        <AlertDialog.Description>
            They are removed from voice_ban.dt. A backup copy of the file is saved beside it first.
        </AlertDialog.Description>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action onclick={confirmUnmute}>Unmute</AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

<AlertDialog.Root bind:open={importOpen}>
    <AlertDialog.Content>
        <AlertDialog.Title
            >Import {newImportIds.length} new mute{newImportIds.length === 1 ? "" : "s"}?</AlertDialog.Title
        >
        <AlertDialog.Description>
            {importIds.length} id{importIds.length === 1 ? "" : "s"} in the file, {importIds.length -
                newImportIds.length} already muted. Existing mutes are kept. A backup copy is saved first.
        </AlertDialog.Description>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action onclick={() => mute(newImportIds)} disabled={newImportIds.length === 0 || locked}
                >Import</AlertDialog.Action
            >
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
