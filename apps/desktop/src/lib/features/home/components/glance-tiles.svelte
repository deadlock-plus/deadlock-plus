<script lang="ts">
    type Props = {
        tiles: { href: string; label: string; value: string }[];
    };

    let { tiles }: Props = $props();
</script>

<section class="grid grid-cols-1 gap-3 sm:grid-cols-3" aria-label="At a glance">
    {#each tiles as tile (tile.href)}
        <a
            href={tile.href}
            class="rounded-lg border border-border bg-card px-5 py-4 text-center transition-colors hover:bg-accent/50"
        >
            <p class="font-heading text-4xl font-semibold tabular-nums text-brass">{tile.value}</p>
            <p class="mt-1 text-sm text-muted-foreground">{tile.label}</p>
        </a>
    {/each}
</section>
