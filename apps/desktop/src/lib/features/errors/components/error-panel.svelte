<script lang="ts">
    import { toast } from "svelte-sonner";
    import { Bug, Copy, RotateCcw, TriangleAlert } from "@lucide/svelte";
    import Button from "$lib/components/ui/button.svelte";
    import { settingsUi } from "$lib/features/settings/ui.svelte";
    import { errorReport, errorSummary } from "../report";

    let { error, route, status, onretry }: { error: unknown; route: string; status?: number; onretry: () => void } =
        $props();

    async function copyDetails() {
        try {
            await navigator.clipboard.writeText(errorReport(error, { route, status }));
            toast.success("Copied error details");
        } catch {
            toast.error("Couldn't copy to the clipboard");
        }
    }
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 p-8 text-center" role="alert">
    <TriangleAlert class="size-10 text-destructive" aria-hidden="true" />
    <div class="flex max-w-md flex-col gap-1">
        <h1 class="font-heading text-lg font-semibold tracking-wide">Something went wrong</h1>
        <p class="text-sm text-muted-foreground">This page hit an error. The rest of the app is still running.</p>
        <p class="mt-2 break-words rounded-md border bg-card px-3 py-2 font-mono text-xs">
            {status ? `${status}: ` : ""}{errorSummary(error)}
        </p>
    </div>
    <div class="flex flex-wrap justify-center gap-2">
        <Button onclick={onretry}>
            <RotateCcw />
            Try again
        </Button>
        <Button variant="outline" onclick={copyDetails}>
            <Copy />
            Copy details
        </Button>
        <Button variant="outline" onclick={() => settingsUi.show("diagnostics")}>
            <Bug />
            Open logs
        </Button>
    </div>
</div>
