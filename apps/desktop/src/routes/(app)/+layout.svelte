<script lang="ts">
    import Frame from "$lib/shell/frame.svelte";
    import Sidebar from "$lib/shell/sidebar.svelte";
    import OfflineBanner from "$lib/features/connectivity/components/offline-banner.svelte";

    let { children } = $props();
</script>

<Frame>
    {#snippet sidebar()}
        <Sidebar />
    {/snippet}
    <OfflineBanner />
    {@render children?.()}
</Frame>
