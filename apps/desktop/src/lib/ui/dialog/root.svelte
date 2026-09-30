<script lang="ts">
    import { Dialog } from "bits-ui";
    import { overlayHost } from "$lib/shell/overlay-host.svelte";

    let { open = $bindable(false), onOpenChange, ...rest }: Dialog.RootProps = $props();

    $effect(() => {
        if (!open) return;
        return overlayHost.track(() => {
            open = false;
            onOpenChange?.(false);
        });
    });
</script>

<Dialog.Root bind:open {onOpenChange} {...rest} />
