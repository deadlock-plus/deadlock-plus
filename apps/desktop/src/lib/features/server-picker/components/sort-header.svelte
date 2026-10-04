<script lang="ts">
    import { ArrowDown, ArrowUp } from "@lucide/svelte";
    import { t } from "$lib/core/i18n.svelte";
    import type { SortKey, SortState } from "../sort";

    let { sort, onSort }: { sort: SortState; onSort: (key: SortKey) => void } = $props();
</script>

{#snippet sortLabel(key: SortKey, label: string, class_: string)}
    {@const active = sort.key === key}
    <button
        type="button"
        onclick={() => onSort(key)}
        aria-label={t("server_picker.sort.by", { label })}
        class="flex items-center gap-1 rounded hover:text-foreground {active ? 'text-foreground' : ''} {class_}"
    >
        {label}
        {#if active}
            {#if sort.dir === "asc"}
                <ArrowUp class="size-3" aria-label={t("server_picker.sort.ascending")} />
            {:else}
                <ArrowDown class="size-3" aria-label={t("server_picker.sort.descending")} />
            {/if}
        {/if}
    </button>
{/snippet}

<div class="flex items-center gap-3 px-4 text-xs font-medium text-muted-foreground">
    <div class="flex-1 pl-8">{@render sortLabel("region", t("server_picker.sort.region"), "")}</div>
    <div class="flex w-20 justify-center">
        {@render sortLabel("ping", t("server_picker.sort.ping"), "whitespace-nowrap")}
    </div>
    <div class="flex w-28 justify-end">{@render sortLabel("blocked", t("server_picker.sort.blocked"), "")}</div>
</div>
