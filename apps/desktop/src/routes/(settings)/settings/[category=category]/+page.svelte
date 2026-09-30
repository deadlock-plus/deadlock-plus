<script lang="ts">
    import { page } from "$app/state";
    import SettingsPage from "$lib/features/settings/components/settings-page.svelte";
    import type { CategoryId } from "$lib/features/settings/catalog";
    import { sidebarState } from "$lib/shell/sidebar-state.svelte";
</script>

<SettingsPage category={page.params.category as CategoryId} collapsed={sidebarState.collapsed} />
