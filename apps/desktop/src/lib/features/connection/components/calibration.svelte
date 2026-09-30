<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import Input from "$lib/ui/input.svelte";
    import Section from "$lib/ui/section.svelte";
    import { formatOffset } from "../connection";

    type Props = {
        entered: string;
        offset: number | null;
        canCalibrate: boolean;
        oncalibrate: () => void;
        onreset: () => void;
    };

    let { entered = $bindable(), offset, canCalibrate, oncalibrate, onreset }: Props = $props();
</script>

<Section title="Calibrate ExitLag estimate" titleClass="">
    <p class="mt-1 text-xs text-muted-foreground">
        The estimate is the exit server's ping plus a fixed last hop to the game server. Enter the ping ExitLag shows
        right now and the offset is computed once and saved.
    </p>
    <div class="mt-3 flex items-center gap-2">
        <Input
            bind:value={entered}
            type="number"
            placeholder="ExitLag shows (ms)"
            aria-label="ExitLag latency in milliseconds"
            class="w-48"
        />
        <Button size="sm" onclick={oncalibrate} disabled={!canCalibrate}>Calibrate</Button>
        {#if offset != null}
            <Button size="sm" variant="ghost" onclick={onreset}>Reset ({formatOffset(offset)} ms)</Button>
        {/if}
    </div>
</Section>
