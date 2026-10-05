<script lang="ts">
    import Input from "$lib/ui/input.svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { t } from "$lib/core/i18n.svelte";
    import type { PresencePartialImage } from "$lib/generated/types/PresencePartialImage";
    import FieldFrame from "./field-frame.svelte";
    import { SOURCE_KINDS, sourceFromKind, sourceKind, sourceUrl, type SourceKind } from "./editor";
    import { SOURCE_LABELS } from "./labels";

    let {
        id,
        label,
        value,
        custom,
        onchange,
        onreset,
    }: {
        id: string;
        label: string;
        value: PresencePartialImage;
        custom: boolean;
        onchange: (patch: PresencePartialImage) => void;
        onreset: () => void;
    } = $props();

    const kind = $derived(sourceKind(value.source) ?? "heroPortrait");
    const url = $derived(sourceUrl(value.source));
    const enabled = $derived(value.enabled ?? false);
</script>

<FieldFrame {id} {label} {custom} {onreset}>
    <div class="flex flex-wrap items-center gap-3">
        <Switch
            id={`${id}-enabled`}
            checked={enabled}
            aria-label={t("settings.discord_editor.image_enabled", { image: label })}
            onCheckedChange={(v) => onchange({ enabled: v })}
        />
        <Select
            {id}
            value={kind}
            disabled={!enabled}
            aria-label={t("settings.discord_editor.image_source", { image: label })}
            onchange={(e) => onchange({ source: sourceFromKind(e.currentTarget.value as SourceKind, url) })}
        >
            {#each kind === "modeIcon" ? [...SOURCE_KINDS, kind] : SOURCE_KINDS as option (option)}
                <option value={option}>{SOURCE_LABELS[option]()}</option>
            {/each}
        </Select>
    </div>
    {#if enabled && kind === "customUrl"}
        <Input
            value={url}
            type="url"
            placeholder="https://"
            aria-label={t("settings.discord_editor.image_url", { image: label })}
            oninput={(e) => onchange({ source: { customUrl: e.currentTarget.value } })}
        />
    {/if}
</FieldFrame>
