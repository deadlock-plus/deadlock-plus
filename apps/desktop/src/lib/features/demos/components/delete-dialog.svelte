<script lang="ts">
    import { TriangleAlert } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import ConfirmDialog from "$lib/ui/confirm-dialog.svelte";
    import { radioTarget } from "$lib/core/radio-group";
    import { formatBytes, type DeleteMode, type DeletePreview } from "$lib/features/demos/demos";
    import { platform, trashName } from "$lib/core/platform";

    const OPTIONS = [
        { mode: "recycle", label: `Move to ${trashName(platform)}`, hint: "You can restore it from there." },
        { mode: "permanent", label: "Delete permanently", hint: "Frees the space now. Can't be undone." },
    ] as const;

    type Copy = { title: string; canRecycle: boolean; notice: string | null };

    let {
        open = $bindable(false),
        mode = $bindable<DeleteMode>("recycle"),
        preview,
        copy,
        deleting,
        onconfirm,
    }: {
        open?: boolean;
        mode?: DeleteMode;
        preview: DeletePreview | null;
        copy: Copy | null;
        deleting: boolean;
        onconfirm: () => void | Promise<void>;
    } = $props();
</script>

<ConfirmDialog
    bind:open
    class="max-w-md"
    title={copy?.title ?? ""}
    destructive={mode === "permanent"}
    disabled={deleting}
    confirmLabel={mode === "permanent" ? "Delete permanently" : `Move to ${trashName(platform)}`}
    {onconfirm}
>
    {#snippet description()}
        {#if preview}Frees {formatBytes(preview.totalBytes)}. Replays aren't backed up in Steam Cloud.{/if}
    {/snippet}
    {#if copy?.notice}
        <div class="flex items-start gap-3 rounded-md border border-warning/40 bg-warning/10 px-3 py-2.5 text-sm">
            <TriangleAlert class="mt-0.5 size-4 shrink-0 text-warning" aria-hidden="true" />
            <p>{copy.notice}</p>
        </div>
    {:else if copy?.canRecycle}
        <div class="flex flex-col gap-2" role="radiogroup" aria-label="Delete method">
            {#each OPTIONS as option (option.mode)}
                {@const on = mode === option.mode}
                <Button
                    variant="unstyled"
                    type="button"
                    role="radio"
                    aria-checked={on}
                    tabindex={on ? 0 : -1}
                    class="flex items-center gap-3 rounded-md border px-3 py-2.5 text-left transition-colors {on
                        ? 'border-primary bg-primary/10'
                        : 'border-border hover:bg-accent/40'}"
                    onclick={() => (mode = option.mode)}
                    onkeydown={(e) => {
                        const at = OPTIONS.findIndex((o) => o.mode === option.mode);
                        const next = radioTarget(e.key, at, OPTIONS.length);
                        if (next === null) return;
                        e.preventDefault();
                        mode = OPTIONS[next].mode;
                        (e.currentTarget.parentElement?.children[next] as HTMLElement | undefined)?.focus();
                    }}
                >
                    <span
                        class="flex size-4 shrink-0 items-center justify-center rounded-full border {on
                            ? 'border-primary'
                            : 'border-muted-foreground/60'}"
                    >
                        {#if on}<span class="size-2 rounded-full bg-primary"></span>{/if}
                    </span>
                    <span class="flex flex-col">
                        <span class="text-sm font-medium">{option.label}</span>
                        <span class="text-xs text-muted-foreground">{option.hint}</span>
                    </span>
                </Button>
            {/each}
        </div>
    {/if}
</ConfirmDialog>
