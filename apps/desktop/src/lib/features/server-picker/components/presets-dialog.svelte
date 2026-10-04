<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import Input from "$lib/ui/input.svelte";
    import IconButton from "$lib/ui/icon-button.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import { radioTarget } from "$lib/core/radio-group";
    import Flag from "$lib/components/flag.svelte";
    import * as Dialog from "$lib/ui/dialog";
    import { Pencil, Plus, Trash2 } from "@lucide/svelte";
    import type { ServerGroup } from "../types";
    import { resolveBlockedIds, type Preset, type PresetMode } from "../presets";

    type Props = {
        open: boolean;
        presets: Preset[];
        regions: ServerGroup[];
        currentlyBlockedRegionIds: string[];
        onApply: (preset: Preset) => void;
        onSave: (preset: Preset) => void;
        onDelete: (id: string) => void;
    };

    let {
        open = $bindable(),
        presets,
        regions,
        currentlyBlockedRegionIds,
        onApply,
        onSave,
        onDelete,
    }: Props = $props();

    const MODES: { mode: PresetMode; title: string; note: string }[] = [
        { mode: "allow", title: "Only allow these", note: "Listed regions stay open, everything else is blocked." },
        { mode: "block", title: "Only block these", note: "Listed regions are blocked, everything else is open." },
    ];

    let editing = $state<Preset | null>(null);
    let filter = $state("");

    const regionIds = $derived(regions.map((r) => r.id));
    const visibleRegions = $derived(regions.filter((r) => r.description.toLowerCase().includes(filter.toLowerCase())));
    const willBlock = $derived(editing ? resolveBlockedIds(editing, regionIds).length : 0);
    const isExisting = $derived(editing != null && presets.some((p) => p.id === editing!.id));

    $effect(() => {
        if (!open) {
            editing = null;
            filter = "";
        }
    });

    function startNew(fromCurrent: boolean) {
        editing = {
            id: crypto.randomUUID(),
            name: "",
            mode: fromCurrent ? "block" : "allow",
            regionIds: fromCurrent ? [...currentlyBlockedRegionIds] : [],
        };
    }

    function toggleRegion(id: string, checked: boolean) {
        if (!editing) return;
        const next = new Set(editing.regionIds);
        if (checked) next.add(id);
        else next.delete(id);
        editing = { ...editing, regionIds: [...next] };
    }

    function summary(p: Preset): string {
        const listed = p.regionIds.filter((id) => regionIds.includes(id)).length;
        const blocked = resolveBlockedIds(p, regionIds).length;
        const what = p.mode === "allow" ? `Only ${listed} open` : `${listed} blocked`;
        return `${what} · blocks ${blocked} of ${regionIds.length} regions`;
    }

    function save() {
        if (!editing || !editing.name.trim()) return;
        onSave({ ...editing, name: editing.name.trim() });
        editing = null;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content>
        {#if editing}
            <Dialog.Title>{isExisting ? "Edit preset" : "New preset"}</Dialog.Title>
            <Dialog.Description>
                Applying a preset sets every region at once. Relay-level blocks inside a region are cleared.
            </Dialog.Description>

            <Input bind:value={editing.name} placeholder="Preset name, e.g. EU only" aria-label="Preset name" />

            <div class="grid grid-cols-2 gap-2" role="radiogroup" aria-label="Preset mode">
                {#each MODES as option, at (option.mode)}
                    <Button
                        variant="unstyled"
                        role="radio"
                        aria-checked={editing.mode === option.mode}
                        tabindex={editing.mode === option.mode ? 0 : -1}
                        onclick={() => editing && (editing = { ...editing, mode: option.mode })}
                        onkeydown={(e) => {
                            const next = radioTarget(e.key, at, MODES.length);
                            if (next === null || !editing) return;
                            e.preventDefault();
                            editing = { ...editing, mode: MODES[next].mode };
                            (e.currentTarget.parentElement?.children[next] as HTMLElement | undefined)?.focus();
                        }}
                        class="rounded-md border px-3 py-2 text-left transition-colors {editing.mode === option.mode
                            ? 'border-primary bg-accent'
                            : 'border-border hover:bg-accent/50'}"
                    >
                        <span class="block text-sm font-medium">{option.title}</span>
                        <span class="block text-xs text-muted-foreground">{option.note}</span>
                    </Button>
                {/each}
            </div>

            <Input bind:value={filter} placeholder="Filter regions..." aria-label="Filter regions" />

            <div class="min-h-0 flex-1 overflow-y-auto rounded-md border border-border">
                {#each visibleRegions as region (region.id)}
                    <label class="flex cursor-pointer items-center gap-3 px-3 py-2 text-sm hover:bg-accent/50">
                        <input
                            type="checkbox"
                            class="size-4"
                            checked={editing.regionIds.includes(region.id)}
                            onchange={(e) => toggleRegion(region.id, e.currentTarget.checked)}
                        />
                        <Flag code={region.countryCode} />
                        <span>{region.description}</span>
                    </label>
                {/each}
            </div>

            <p class="text-xs text-muted-foreground">
                {willBlock} of {regionIds.length} regions will be blocked, {regionIds.length - willBlock} left open.
            </p>

            <div class="flex justify-end gap-2">
                <Button variant="outline" onclick={() => (editing = null)}>Back</Button>
                <Button onclick={save} disabled={!editing.name.trim()}>Save preset</Button>
            </div>
        {:else}
            <Dialog.Title>Presets</Dialog.Title>
            <Dialog.Description>
                Save a set of regions to block or keep open, then apply it in one click. Useful for region-locking, like
                an EU-only preset.
            </Dialog.Description>

            <div class="min-h-0 flex-1 overflow-y-auto">
                <div class="flex flex-col gap-2">
                    {#each presets as preset (preset.id)}
                        <div class="flex items-center gap-3 rounded-md border border-border px-3 py-2">
                            <div class="min-w-0 flex-1">
                                <div class="truncate text-sm font-medium">{preset.name}</div>
                                <div class="text-xs text-muted-foreground">{summary(preset)}</div>
                            </div>
                            <Button size="sm" onclick={() => onApply(preset)}>Apply</Button>
                            <IconButton label="Edit {preset.name}" onclick={() => (editing = { ...preset })}>
                                <Pencil />
                            </IconButton>
                            <IconButton label="Delete {preset.name}" onclick={() => onDelete(preset.id)}>
                                <Trash2 />
                            </IconButton>
                        </div>
                    {:else}
                        <EmptyState spacing="sm">No presets yet.</EmptyState>
                    {/each}
                </div>
            </div>

            <div class="flex flex-wrap justify-end gap-2">
                <Button
                    variant="outline"
                    onclick={() => startNew(true)}
                    disabled={currentlyBlockedRegionIds.length === 0}
                >
                    Save current blocks
                </Button>
                <Button onclick={() => startNew(false)}><Plus /> New preset</Button>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>
