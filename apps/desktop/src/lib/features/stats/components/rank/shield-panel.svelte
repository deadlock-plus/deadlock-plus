<script lang="ts">
    import { Shield, ShieldOff } from "@lucide/svelte";

    import { t, tn } from "$lib/core/i18n.svelte";

    let { shieldsLeft }: { shieldsLeft: number | null } = $props();
</script>

<div
    class="flex flex-col items-center justify-center gap-2 rounded-md border border-border bg-background/40 px-6 py-4 md:min-w-52"
>
    <p class="text-sm text-muted-foreground">{t("rank.shields.heading")}</p>
    {#if shieldsLeft === null}
        <p class="text-sm text-muted-foreground">{t("rank.shields.not_reported")}</p>
    {:else if shieldsLeft === 0}
        <ShieldOff class="size-9 text-destructive" aria-hidden="true" />
        <p class="font-heading text-lg">{t("rank.shields.none_left")}</p>
        <p class="text-center text-xs text-muted-foreground">{t("rank.shields.none_hint")}</p>
    {:else}
        <div class="flex gap-1.5" aria-hidden="true">
            {#each { length: shieldsLeft } as _, i (i)}
                <Shield class="size-9 fill-primary/25 text-primary" />
            {/each}
        </div>
        <p class="font-heading text-lg">{tn("rank.shields.left", shieldsLeft)}</p>
        <p class="text-center text-xs text-muted-foreground">{t("rank.shields.left_hint")}</p>
    {/if}
</div>
