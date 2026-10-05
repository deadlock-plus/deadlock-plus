<script lang="ts">
    import { tick } from "svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { t } from "$lib/core/i18n.svelte";
    import { effectiveSlot, getSlot, resetField, setField, type Scope } from "$lib/features/presence/config";
    import type { PresenceConfig } from "$lib/generated/types/PresenceConfig";
    import type { PresencePlaceholder } from "$lib/generated/types/PresencePlaceholder";
    import type { PresencePartialSlot } from "$lib/generated/types/PresencePartialSlot";
    import type { PresenceTimer } from "$lib/generated/types/PresenceTimer";
    import FieldFrame from "./field-frame.svelte";
    import ImageField from "./image-field.svelte";
    import TextField from "./text-field.svelte";
    import VariablesMenu from "./variables-menu.svelte";
    import { TIMERS, insertPlaceholder } from "./editor";
    import { TIMER_LABELS } from "./labels";

    let {
        scope,
        config,
        defaults,
        placeholders,
        edit,
    }: {
        scope: Scope;
        config: PresenceConfig;
        defaults: PresenceConfig;
        placeholders: PresencePlaceholder[];
        edit: (next: PresenceConfig) => void;
    } = $props();

    const own = $derived(getSlot(config, scope));
    const shown = $derived(effectiveSlot(config, defaults, scope));
    const idBase = $derived(`pe-${scope.state}-${scope.variant ?? "base"}-${scope.heroId ?? "all"}`);

    const set = (patch: PresencePartialSlot) => edit(setField(config, scope, patch));
    const reset = (field: keyof PresencePartialSlot) => edit(resetField(config, scope, field));

    type TextKey = "details" | "state" | "largeText" | "smallText";
    const TEXT_IDS: Record<TextKey, string> = {
        details: "details",
        state: "state",
        largeText: "large-text",
        smallText: "small-text",
    };
    const TEXT_LABELS: Record<TextKey, () => string> = {
        details: () => t("settings.discord_editor.field_details"),
        state: () => t("settings.discord_editor.field_state"),
        largeText: () => t("settings.discord_editor.field_large_text"),
        smallText: () => t("settings.discord_editor.field_small_text"),
    };
    let activeField = $state<TextKey>("details");

    async function insertVariable(name: string) {
        const key = activeField;
        const input = document.getElementById(`${idBase}-${TEXT_IDS[key]}`) as HTMLInputElement | null;
        if (!input) return;
        const next = insertPlaceholder(shown[key] ?? "", input.selectionStart, input.selectionEnd, name);
        set({ [key]: next.text });
        await tick();
        input.focus();
        input.setSelectionRange(next.caret, next.caret);
    }
</script>

<div class="flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-2">
        <p class="text-xs text-muted-foreground">{t("settings.discord_editor.variables_hint")}</p>
        <VariablesMenu {placeholders} target={TEXT_LABELS[activeField]()} insert={insertVariable} />
    </div>

    <TextField
        id={`${idBase}-details`}
        label={t("settings.discord_editor.field_details")}
        value={shown.details ?? ""}
        custom={own.details !== undefined}
        onchange={(v) => set({ details: v })}
        onfocus={() => (activeField = "details")}
        onreset={() => reset("details")}
    />
    <TextField
        id={`${idBase}-state`}
        label={t("settings.discord_editor.field_state")}
        value={shown.state ?? ""}
        custom={own.state !== undefined}
        onchange={(v) => set({ state: v })}
        onfocus={() => (activeField = "state")}
        onreset={() => reset("state")}
    />

    <ImageField
        id={`${idBase}-large-image`}
        label={t("settings.discord_editor.field_large_image")}
        value={shown.largeImage ?? {}}
        custom={own.largeImage !== undefined}
        onchange={(patch) => set({ largeImage: patch })}
        onreset={() => reset("largeImage")}
    />
    <TextField
        id={`${idBase}-large-text`}
        label={t("settings.discord_editor.field_large_text")}
        value={shown.largeText ?? ""}
        custom={own.largeText !== undefined}
        onchange={(v) => set({ largeText: v })}
        onfocus={() => (activeField = "largeText")}
        onreset={() => reset("largeText")}
    />

    <ImageField
        id={`${idBase}-small-image`}
        label={t("settings.discord_editor.field_small_image")}
        value={shown.smallImage ?? {}}
        custom={own.smallImage !== undefined}
        onchange={(patch) => set({ smallImage: patch })}
        onreset={() => reset("smallImage")}
    />
    <TextField
        id={`${idBase}-small-text`}
        label={t("settings.discord_editor.field_small_text")}
        value={shown.smallText ?? ""}
        custom={own.smallText !== undefined}
        onchange={(v) => set({ smallText: v })}
        onfocus={() => (activeField = "smallText")}
        onreset={() => reset("smallText")}
    />

    <FieldFrame
        id={`${idBase}-party-size`}
        label={t("settings.discord_editor.field_party_size")}
        custom={own.partySize !== undefined}
        onreset={() => reset("partySize")}
    >
        <div class="flex items-center gap-3">
            <Switch
                id={`${idBase}-party-size`}
                checked={shown.partySize ?? true}
                onCheckedChange={(v) => set({ partySize: v })}
            />
            <span class="text-xs text-muted-foreground">{t("settings.discord_editor.party_size_hint")}</span>
        </div>
    </FieldFrame>

    <FieldFrame
        id={`${idBase}-timer`}
        label={t("settings.discord_editor.field_timer")}
        custom={own.timer !== undefined}
        onreset={() => reset("timer")}
    >
        <Select
            id={`${idBase}-timer`}
            class="w-fit"
            value={shown.timer ?? "none"}
            onchange={(e) => set({ timer: e.currentTarget.value as PresenceTimer })}
        >
            {#each TIMERS as timer (timer)}
                <option value={timer}>{TIMER_LABELS[timer]()}</option>
            {/each}
        </Select>
    </FieldFrame>
</div>
