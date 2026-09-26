<script lang="ts">
    import LogViewer from "$lib/features/settings/components/log-viewer.svelte";

    let { show }: { show: (id: string) => boolean } = $props();
</script>

{#if show("diagnostics")}
    <LogViewer />
{/if}
