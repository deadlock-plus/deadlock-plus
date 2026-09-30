<script lang="ts">
    import { AlertDialog } from "bits-ui";
    import { cn } from "$lib/core/utils";
    import { overlayHost } from "$lib/shell/overlay-host.svelte";

    let { class: className, ...rest }: AlertDialog.ContentProps = $props();
</script>

{#if overlayHost.el}
    <AlertDialog.Portal to={overlayHost.el}>
        <AlertDialog.Overlay
            class="absolute inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
        />
        <AlertDialog.Content
            trapFocus={false}
            preventScroll={false}
            class={cn(
                "absolute left-1/2 top-1/2 grid max-h-[calc(100%-2rem)] w-full max-w-sm -translate-x-1/2 -translate-y-1/2 gap-4 overflow-y-auto rounded-lg border border-border bg-card p-6 shadow-lg duration-150 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
                className,
            )}
            {...rest}
        />
    </AlertDialog.Portal>
{/if}
