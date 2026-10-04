<script lang="ts">
    import type { Snippet } from "svelte";
    import { LoaderCircle } from "@lucide/svelte";

    import EmptyState from "$lib/ui/empty-state.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { stats } from "../../stats.svelte";

    let { children }: { children: Snippet } = $props();

    const accountId = $derived(steamAccount.account?.steamId32 ?? null);
</script>

{#if steamAccount.loaded && accountId === null}
    <EmptyState size="base" spacing="xl">No Steam account found, so there is no history to load.</EmptyState>
{:else if stats.status === "error"}
    <EmptyState size="base" spacing="xl" tone="destructive" role="alert"
        >Could not load your match history. {stats.error}</EmptyState
    >
{:else if stats.status !== "ready"}
    <EmptyState size="base" spacing="xl" role="status" class="flex items-center justify-center gap-2">
        <LoaderCircle class="size-4 animate-spin" aria-hidden="true" /> Loading your match history...
    </EmptyState>
{:else}
    {@render children()}
{/if}
