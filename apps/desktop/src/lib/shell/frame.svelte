<script lang="ts">
    import type { Snippet } from "svelte";
    import ContentRegion from "./content-region.svelte";
    import Statusbar from "./statusbar.svelte";

    type Props = {
        sidebar?: Snippet;
        statusbar?: boolean;
        children?: Snippet;
    };

    let { sidebar, statusbar = true, children }: Props = $props();
</script>

<div class="flex min-h-0 flex-1 flex-col">
    <div class="flex min-h-0 flex-1">
        {@render sidebar?.()}
        <ContentRegion flush={!sidebar}>{@render children?.()}</ContentRegion>
    </div>
    {#if statusbar}
        <Statusbar />
    {/if}
</div>
