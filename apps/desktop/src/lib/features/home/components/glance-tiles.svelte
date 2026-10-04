<script lang="ts">
    import { Play } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
    import { gameTile } from "$lib/features/home/home";

    type Props = {
        tiles: { href: string; label: string; value: string }[];
        gameRunning: boolean | null;
        onLaunch: () => void;
    };

    let { tiles, gameRunning, onLaunch }: Props = $props();

    const game = $derived(gameTile(gameRunning));
    const tileClass = "rounded-lg border border-border bg-card px-5 py-4 text-center";
</script>

<section class="grid grid-cols-1 gap-3 sm:grid-cols-4" aria-label={t("home.glance_label")}>
    {#if game.launchable}
        <button type="button" onclick={onLaunch} class="{tileClass} transition-colors hover:bg-accent/50">
            <p class="flex h-10 items-center justify-center gap-2 font-heading text-3xl font-semibold text-brass">
                <Play class="size-6" />{game.label}
            </p>
            <p class="mt-1 text-sm text-muted-foreground">Deadlock</p>
        </button>
    {:else}
        <div class={tileClass}>
            <p class="flex h-10 items-center justify-center font-heading text-3xl font-semibold text-success">
                {game.label}
            </p>
            <p class="mt-1 text-sm text-muted-foreground">Deadlock</p>
        </div>
    {/if}
    {#each tiles as tile (tile.href)}
        <a href={tile.href} class="{tileClass} transition-colors hover:bg-accent/50">
            <p class="font-heading text-4xl font-semibold tabular-nums text-brass">{tile.value}</p>
            <p class="mt-1 text-sm text-muted-foreground">{tile.label}</p>
        </a>
    {/each}
</section>
