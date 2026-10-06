<script lang="ts">
    import type { Hero } from "$lib/features/heroes/heroes";
    import { heroInitials } from "../live";

    let {
        heroId,
        heroes,
        size = 22,
    }: { heroId: number | null; heroes: Record<number, Hero>; size?: number } = $props();

    const hero = $derived(heroId === null ? null : (heroes[heroId] ?? null));
    const hue = $derived(((heroId ?? 0) * 47) % 360);
</script>

{#if hero?.icon}
    <img
        src={hero.icon}
        alt={hero.name}
        title={hero.name}
        class="hero"
        style:width="{size}px"
        style:height="{size}px"
    />
{:else}
    <span
        class="hero initials"
        title={hero?.name}
        style:width="{size}px"
        style:height="{size}px"
        style:background="oklch(0.72 0.07 {hue})"
    >
        {heroInitials(hero?.name ?? null)}
    </span>
{/if}

<style>
    .hero {
        display: inline-flex;
        flex: none;
        align-items: center;
        justify-content: center;
        border-radius: 6px;
        object-fit: cover;
    }
    .initials {
        font-size: 10px;
        font-weight: 700;
        color: oklch(0.15 0.01 70);
    }
</style>
