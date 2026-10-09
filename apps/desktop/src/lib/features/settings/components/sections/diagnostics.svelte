<script lang="ts">
    import LogViewer from "$lib/features/settings/components/log-viewer.svelte";
    import SupportReport from "$lib/features/settings/components/support-report.svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";

    let { show }: { show: (id: string) => boolean } = $props();
</script>

{#if show("diagnostics")}
    {#if !settingsUi.logsExpanded}
        <SupportReport />
    {/if}
    <LogViewer />
{/if}
