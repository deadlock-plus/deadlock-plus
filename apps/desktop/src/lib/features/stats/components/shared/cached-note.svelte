<script lang="ts">
    import { formatDate, t } from "$lib/core/i18n.svelte";
    import { stats } from "../../stats.svelte";
</script>

{#if stats.cachedAt !== null}
    <p class="text-sm text-muted-foreground">
        {t("stats.cached_note", {
            time: formatDate(stats.cachedAt, {
                year: "numeric",
                month: "numeric",
                day: "numeric",
                hour: "numeric",
                minute: "numeric",
                second: "numeric",
            }),
        })}
    </p>
{/if}
