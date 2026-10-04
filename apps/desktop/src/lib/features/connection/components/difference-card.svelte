<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import { formatNumber, t } from "$lib/core/i18n.svelte";

    type Props = {
        saved: number | null;
    };

    let { saved }: Props = $props();
</script>

<Card class="flex flex-col gap-3">
    <span class="text-sm font-medium">{t("connection.difference.title")}</span>
    {#if saved == null}
        <p class="py-6 text-sm text-muted-foreground">{t("connection.difference.needs_both")}</p>
    {:else}
        <div class="flex items-baseline gap-1">
            <span class="text-4xl font-semibold tabular-nums {saved >= 0 ? 'text-success' : 'text-destructive'}">
                {saved >= 0 ? "−" : "+"}{formatNumber(Math.abs(Math.round(saved)))}
            </span>
            <span class="text-sm text-muted-foreground">{t("connection.unit_ms")}</span>
        </div>
        <p class="text-xs text-muted-foreground">
            {saved >= 0 ? t("connection.difference.faster") : t("connection.difference.slower")}
        </p>
    {/if}
</Card>
