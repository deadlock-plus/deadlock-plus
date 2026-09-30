<script lang="ts">
    import { Dialog } from "bits-ui";
    import { cn } from "$lib/core/utils";
    import { overlayHost } from "$lib/shell/overlay-host.svelte";

    let { class: className, ...rest }: Dialog.ContentProps = $props();

    // Clicks on the titlebar, sidebar or status bar are not "outside" the dialog: those stay live.
    function keepChromeOpen(e: PointerEvent) {
        const host = overlayHost.el;
        if (host && e.target instanceof Node && !host.contains(e.target)) e.preventDefault();
    }
</script>

{#if overlayHost.el}
    <Dialog.Portal to={overlayHost.el}>
        <Dialog.Overlay
            class="absolute inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
        />
        <Dialog.Content
            trapFocus={false}
            preventScroll={false}
            interactOutsideBehavior="close"
            onInteractOutside={keepChromeOpen}
            class={cn(
                "absolute left-1/2 top-1/2 flex max-h-[calc(100%-2rem)] w-[calc(100%-2rem)] max-w-xl -translate-x-1/2 -translate-y-1/2 flex-col gap-4 rounded-lg border border-border bg-card p-6 shadow-lg duration-150 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
                className,
            )}
            {...rest}
        />
    </Dialog.Portal>
{/if}
