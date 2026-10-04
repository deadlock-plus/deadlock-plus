<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import { t } from "$lib/core/i18n.svelte";
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

<Section title={t("connection.calibration.title")} titleClass="">
    <p class="mt-1 text-xs text-muted-foreground">
        {t("connection.calibration.help")}
    </p>
    <div class="mt-3 flex items-center gap-2">
        <Input
            bind:value={entered}
            type="number"
            placeholder={t("connection.calibration.placeholder")}
            aria-label={t("connection.calibration.input_label")}
            class="w-48"
            onkeydown={(e) => {
                if (e.key === "Enter" && canCalibrate) oncalibrate();
            }}
        />
        <Button size="sm" onclick={oncalibrate} disabled={!canCalibrate}>
            {t("connection.calibration.calibrate")}
        </Button>
        {#if offset != null}
            <Button size="sm" variant="ghost" onclick={onreset}>
                {t("connection.calibration.reset", { offset: formatOffset(offset) })}
            </Button>
        {/if}
    </div>
</Section>
