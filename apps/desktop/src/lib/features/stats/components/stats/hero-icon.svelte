<script lang="ts">
    import { CircleHelp } from "@lucide/svelte";

    import type { Hero } from "$lib/features/heroes/heroes";

    let { hero, size }: { hero: Hero | undefined; size: string } = $props();
</script>

<div class="flex shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted {size}">
    {#if hero?.icon}
        <img src={hero.icon} alt="" class="size-full object-cover" />
    {:else}
        <CircleHelp class="size-6 text-muted-foreground" aria-label="Unknown hero" />
    {/if}
</div>
