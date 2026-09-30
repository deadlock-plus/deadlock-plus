<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import Frame from "$lib/shell/frame.svelte";
    import Sidebar from "$lib/shell/sidebar.svelte";
    import ErrorPanel from "$lib/features/errors/components/error-panel.svelte";
</script>

<Frame>
    {#snippet sidebar()}
        <Sidebar />
    {/snippet}
    <ErrorPanel
        error={page.error}
        route={page.url.pathname}
        status={page.status}
        onretry={() => goto("/", { invalidateAll: true })}
    />
</Frame>
