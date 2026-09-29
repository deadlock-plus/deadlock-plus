<script lang="ts">
    import { AlertDialog } from "bits-ui";
    import { cn } from "$lib/utils";

    let { class: className, ...rest }: AlertDialog.ContentProps = $props();
</script>

<AlertDialog.Portal>
    <AlertDialog.Overlay
        class="fixed inset-0 z-50 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
    />
    <AlertDialog.Content
        class={cn(
            "fixed left-1/2 top-1/2 z-50 grid w-full max-w-sm -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border border-border bg-card p-6 shadow-lg duration-150 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
            className,
        )}
        {...rest}
    />
</AlertDialog.Portal>
