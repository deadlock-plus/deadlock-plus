<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import { formatDate, formatNumber, t } from "$lib/core/i18n.svelte";

    import { matchHistory } from "../match-history.svelte";

    const ms = (v: number) => t("connection.ms", { value: formatNumber(Math.round(v)) });
</script>

{#if matchHistory.matches.length > 0}
    <Card as="section">
        <h2 class="mb-2 text-sm font-medium">{t("connection.matches.title")}</h2>
        <ul class="divide-y divide-border text-sm">
            {#each matchHistory.matches as m (m.id)}
                <li class="flex items-center justify-between gap-4 py-2">
                    <div class="min-w-0">
                        <div class="truncate">
                            {formatDate(m.startedAt, { dateStyle: "medium", timeStyle: "short" })}
                        </div>
                        {#if m.server}
                            <div class="truncate text-xs text-muted-foreground">{m.server}</div>
                        {/if}
                    </div>
                    <dl class="flex shrink-0 gap-6 text-xs">
                        <div>
                            <dt class="text-muted-foreground">{t("connection.matches.avg")}</dt>
                            <dd class="tabular-nums">{ms(m.avg)}</dd>
                        </div>
                        <div>
                            <dt class="text-muted-foreground">{t("connection.matches.worst")}</dt>
                            <dd class="tabular-nums">{ms(m.worst)}</dd>
                        </div>
                    </dl>
                </li>
            {/each}
        </ul>
        <p class="mt-2 text-xs text-muted-foreground">{t("connection.matches.note")}</p>
    </Card>
{/if}
