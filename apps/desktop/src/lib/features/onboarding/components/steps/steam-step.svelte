<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
</script>

<h1 class="font-heading text-2xl font-bold tracking-wide">Your Steam account</h1>
<p class="text-muted-foreground">Stats, Rank and Sessions read your match history, so they need your Steam account.</p>

<Card padding="sm" class="flex items-center gap-3 text-sm">
    {#if steamAccount.account}
        {#if steamAccount.account.avatarDataUrl}
            <img src={steamAccount.account.avatarDataUrl} alt="" class="size-10 rounded-md" />
        {/if}
        <div class="flex flex-col">
            <span class="font-semibold">{steamAccount.account.personaName}</span>
            <span class="text-muted-foreground">Signed in on this PC. Nothing to set up.</span>
        </div>
    {:else if steamAccount.loaded}
        <div class="flex flex-col">
            <span class="font-semibold">No Steam account found</span>
            <span class="text-muted-foreground">
                Sign in to Steam on this PC, then reopen Deadlock+. Everything else works without it.
            </span>
        </div>
    {:else}
        <span class="text-muted-foreground">Looking for your Steam account...</span>
    {/if}
</Card>
