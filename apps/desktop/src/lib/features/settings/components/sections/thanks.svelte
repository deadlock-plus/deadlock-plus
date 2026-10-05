<script lang="ts">
    import ThanksCard from "$lib/features/thanks/thanks-card.svelte";

    let { show }: { show: (id: string) => boolean } = $props();
</script>

{#if show("thanks")}
    <ThanksCard />
{/if}
