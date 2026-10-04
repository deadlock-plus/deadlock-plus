<script lang="ts">
    import { WifiOff } from "@lucide/svelte";
    import { t } from "$lib/core/i18n.svelte";
    import { connectivity } from "../online.svelte";
</script>

{#if !connectivity.online}
    <div
        role="status"
        class="flex items-center gap-2 border-b border-border bg-muted px-4 py-2 text-sm text-muted-foreground"
    >
        <WifiOff class="size-4 shrink-0" />
        {t("connectivity.offline")}
    </div>
{/if}
