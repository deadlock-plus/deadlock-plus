<script lang="ts">
    import { platformName, type Platform } from "$lib/core/platform";

    let { platform, elevated }: { platform: Platform; elevated: boolean } = $props();
</script>

{#if platform !== "windows"}
    <h1 class="font-heading text-2xl font-bold tracking-wide">Running on {platformName(platform)}</h1>
    <p class="text-muted-foreground">{platformName(platform)} support is best-effort and mostly untested.</p>
    <ul class="flex list-disc flex-col gap-1.5 pl-5 text-sm text-muted-foreground">
        <li>
            <span class="text-foreground">Not available yet:</span> Frametimes.
        </li>
        <li>
            <span class="text-foreground">Untested:</span> the Server Picker and the Connection page. Both ask for your password.
        </li>
        <li>
            <span class="text-foreground">Something broken?</span> Please report it on GitHub. A fix is a big plus.
        </li>
    </ul>
{:else}
    <h1 class="font-heading text-2xl font-bold tracking-wide">Why Windows asks for permission</h1>
    <p class="text-muted-foreground">Deadlock+ runs as administrator, so Windows shows a UAC prompt on every launch.</p>
    <ul class="flex list-disc flex-col gap-1.5 pl-5 text-sm text-muted-foreground">
        <li>
            <span class="text-foreground">Firewall rules:</span> the Server Picker blocks the regions you pick by adding
            Windows Firewall rules named <code>deadlock_plus_*</code>. Only administrators can do that.
        </li>
        <li>
            <span class="text-foreground">Connection monitor:</span> live ping and loss for your match come from Windows network
            tracing, which is also admin-only.
        </li>
        <li>Nothing else needs elevated rights.</li>
    </ul>
    {#if !elevated}
        <p class="text-sm text-muted-foreground">
            This copy runs without administrator rights, so the Server Picker and Connection page won't work until you
            relaunch it elevated.
        </p>
    {/if}
{/if}
