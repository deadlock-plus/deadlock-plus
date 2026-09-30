<script lang="ts">
    import { AlertDialog } from "bits-ui";
    import { overlayHost } from "$lib/shell/overlay-host.svelte";

    let { open = $bindable(false), onOpenChange, ...rest }: AlertDialog.RootProps = $props();

    $effect(() => {
        if (!open) return;
        return overlayHost.track(() => {
            open = false;
            onOpenChange?.(false);
        });
    });
</script>

<AlertDialog.Root bind:open {onOpenChange} {...rest} />
