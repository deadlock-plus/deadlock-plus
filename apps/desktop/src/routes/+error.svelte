<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import ErrorPanel from "$lib/features/errors/components/error-panel.svelte";
</script>

<ErrorPanel
    error={page.error}
    route={page.url.pathname}
    status={page.status}
    onretry={() => goto("/", { invalidateAll: true })}
/>
