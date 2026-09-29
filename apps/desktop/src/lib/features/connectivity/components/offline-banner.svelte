<script lang="ts">
    import { WifiOff } from "@lucide/svelte";
    import { connectivity } from "../online.svelte";
</script>

{#if !connectivity.online}
    <div
        role="status"
        class="flex items-center gap-2 border-b border-border bg-muted px-4 py-2 text-sm text-muted-foreground"
    >
        <WifiOff class="size-4 shrink-0" />
        You are offline. Deadlock needs a connection, so live data is paused and saved copies are shown where they exist.
    </div>
{/if}
