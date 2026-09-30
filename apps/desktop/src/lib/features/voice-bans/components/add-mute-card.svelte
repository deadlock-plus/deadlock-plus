<script lang="ts">
    import { ExternalLink, Plus } from "@lucide/svelte";
    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import IconButton from "$lib/ui/icon-button.svelte";
    import Input from "$lib/ui/input.svelte";
    import { parseSteamId } from "../voice-ban";
    import type { Profile } from "../profiles";

    let {
        value = $bindable(),
        results,
        mutedSet,
        busy,
        locked,
        searching,
        onsubmit,
        onmute,
        onopen,
    }: {
        value: string;
        results: Profile[] | null;
        mutedSet: Set<string>;
        busy: boolean;
        locked: boolean;
        searching: boolean;
        onsubmit: () => void;
        onmute: (id: string) => void;
        onopen: (id: string) => void;
    } = $props();
</script>

<Card radius="md" padding="sm" class="flex flex-col gap-2">
    <div class="flex gap-2">
        <Input
            bind:value
            placeholder="Mute by SteamID64, account ID, /profiles/ link or player name"
            aria-label="Player to mute"
            onkeydown={(e) => e.key === "Enter" && onsubmit()}
        />
        <Button onclick={onsubmit} disabled={busy || locked || searching || !value.trim()}>
            <Plus />
            {parseSteamId(value) ? "Mute" : "Search"}
        </Button>
    </div>
    {#if results}
        {#if results.length === 0}
            <p class="text-sm text-muted-foreground">No players found for that name.</p>
        {:else}
            <ul class="flex flex-col gap-1">
                {#each results as p (p.steamid64)}
                    <li class="flex items-center gap-3 rounded-md px-2 py-1.5 hover:bg-accent/40">
                        {#if p.avatar}<img src={p.avatar} alt="" class="size-8 rounded-sm" />{/if}
                        <div class="min-w-0 flex-1">
                            <p class="truncate text-sm font-medium">{p.name}</p>
                            <p class="text-xs text-muted-foreground">{p.steamid64}</p>
                        </div>
                        <IconButton
                            size="sm"
                            label="Open Statlocker profile"
                            title="Statlocker profile"
                            onclick={() => onopen(p.steamid64)}
                        >
                            <ExternalLink />
                        </IconButton>
                        {#if mutedSet.has(p.steamid64)}
                            <Badge variant="secondary">Muted</Badge>
                        {:else}
                            <Button
                                size="sm"
                                variant="outline"
                                disabled={busy || locked}
                                onclick={() => onmute(p.steamid64)}>Mute</Button
                            >
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    {/if}
</Card>
