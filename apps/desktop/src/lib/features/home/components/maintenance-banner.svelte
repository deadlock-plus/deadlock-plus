<script lang="ts">
    import { Wrench } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";

    type Props = {
        line: string;
        reminderOn: boolean;
        onopen: () => void;
    };

    let { line, reminderOn, onopen }: Props = $props();
</script>

<Button
    type="button"
    variant="unstyled"
    onclick={onopen}
    class="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3 hover:bg-accent/50"
>
    <Wrench class="size-4 shrink-0 text-muted-foreground" />
    <div class="min-w-0">
        <p class="font-heading text-sm font-semibold tracking-wide">Steam maintenance</p>
        <p class="text-xs text-muted-foreground">
            {line} Valve publishes no schedule, so this is an estimate.
            {reminderOn ? "The reminder is on." : "The reminder is off."}
        </p>
    </div>
</Button>
