<script lang="ts">
    import { t } from "$lib/core/i18n.svelte";
</script>

<div class="flex flex-col gap-2" role="status" aria-live="polite">
    <span class="sr-only">{t("stats.gate.loading")}</span>
    {#each { length: 8 } as _, i (i)}
        <div
            class="flex h-[4.5rem] items-center gap-4 rounded-lg border border-l-4 border-border border-l-muted bg-card px-3 motion-safe:animate-pulse"
            style:opacity={1 - i * 0.09}
            aria-hidden="true"
        >
            <div class="size-12 rounded-lg bg-muted"></div>
            <div class="flex flex-1 flex-col gap-2">
                <div class="h-3.5 w-32 rounded bg-muted"></div>
                <div class="h-3 w-48 rounded bg-muted/70"></div>
            </div>
            <div class="h-5 w-14 rounded bg-muted"></div>
            <div class="h-8 w-20 rounded bg-muted/70"></div>
            <div class="h-5 w-16 rounded bg-muted/70"></div>
        </div>
    {/each}
</div>
