<script lang="ts">
    let { src, label, size = 28 }: { src?: string; label: string; size?: number } = $props();
</script>

{#if src}
    <img
        {src}
        alt={label}
        title={label}
        class="shrink-0 rounded object-contain"
        style:width="{size}px"
        style:height="{size}px"
    />
{:else}
    <span
        class="inline-flex shrink-0 items-center justify-center rounded border border-border bg-muted"
        style:width="{size}px"
        style:height="{size}px"
        title={label}
        aria-hidden="true"
    ></span>
{/if}
