<script lang="ts">
    import { Maximize2, Minimize2 } from "@lucide/svelte";
    import Button from "$lib/components/ui/button.svelte";
    import LogView from "$lib/features/logging/components/log-view.svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";

    const expanded = $derived(settingsUi.logsExpanded);

    // Expanding remounts this component; without this the dialog's focus trap
    // falls back to the first sidebar button and pops its tooltip.
    function restoreFocus(node: HTMLElement) {
        if (!settingsUi.restoreLogsToggleFocus) return;
        settingsUi.restoreLogsToggleFocus = false;
        node.focus();
    }
</script>

<section class="rounded-lg border bg-card p-4 {expanded ? 'flex h-full min-h-0 flex-col' : ''}">
    <div class="flex items-start justify-between gap-4">
        <div>
            <h2 class="font-heading text-sm font-semibold tracking-wide">Diagnostics</h2>
            <p class="mt-1 text-sm text-muted-foreground">
                This session's log. Names in file paths are hidden when you copy or save it. Only needed when reporting
                a bug.
            </p>
        </div>
        <Button
            variant="outline"
            size="sm"
            class="shrink-0"
            onclick={() => settingsUi.toggleLogsExpanded()}
            {@attach restoreFocus}
        >
            {#if expanded}
                <Minimize2 />
                Collapse
            {:else}
                <Maximize2 />
                Expand
            {/if}
        </Button>
    </div>
    <div class="mt-3 {expanded ? 'min-h-0 flex-1' : ''}">
        <LogView full={expanded} />
    </div>
</section>
