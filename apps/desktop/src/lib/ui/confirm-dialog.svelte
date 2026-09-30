<script lang="ts">
    import type { Snippet } from "svelte";
    import * as AlertDialog from "$lib/ui/alert-dialog";
    import { closeOutcome } from "./confirm-dialog";

    type Props = {
        open?: boolean;
        title: string;
        description?: string | Snippet;
        confirmLabel?: string;
        cancelLabel?: string;
        destructive?: boolean;
        disabled?: boolean;
        class?: string;
        children?: Snippet;
        onconfirm?: () => void | Promise<void>;
        oncancel?: () => void;
    };

    let {
        open = $bindable(false),
        title,
        description,
        confirmLabel = "Confirm",
        cancelLabel = "Cancel",
        destructive = false,
        disabled = false,
        class: className,
        children,
        onconfirm,
        oncancel,
    }: Props = $props();

    let confirmed = false;

    function onOpenChange(next: boolean) {
        if (next) confirmed = false;
        else if (closeOutcome(next, confirmed) === "cancel") oncancel?.();
    }
</script>

<AlertDialog.Root bind:open {onOpenChange}>
    <AlertDialog.Content class={className}>
        <div class="flex flex-col gap-1.5">
            <AlertDialog.Title>{title}</AlertDialog.Title>
            {#if typeof description === "string"}
                <AlertDialog.Description>{description}</AlertDialog.Description>
            {:else if description}
                <AlertDialog.Description>{@render description()}</AlertDialog.Description>
            {/if}
        </div>
        {@render children?.()}
        <AlertDialog.Footer>
            <AlertDialog.Cancel>{cancelLabel}</AlertDialog.Cancel>
            <AlertDialog.Action
                variant={destructive ? "destructive" : "default"}
                {disabled}
                onclick={() => {
                    confirmed = true;
                    void onconfirm?.();
                }}
            >
                {confirmLabel}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
