<script lang="ts">
    import { stats } from "../../stats.svelte";
</script>

{#if stats.cachedAt !== null}
    <p class="text-sm text-muted-foreground">
        Could not reach the API. Showing the copy saved {new Date(stats.cachedAt).toLocaleString()}.
    </p>
{/if}
