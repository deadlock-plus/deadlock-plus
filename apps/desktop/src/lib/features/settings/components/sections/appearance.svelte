<script lang="ts">
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { THEMES, resolveMotionPreference } from "$lib/features/settings/themes";

    let { show }: { show: (id: string) => boolean } = $props();
</script>

{#if show("theme")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1">
                <h3 class="font-heading text-sm font-semibold tracking-wide">Theme</h3>
                <p class="text-sm text-muted-foreground">Pick the colours the app uses.</p>
            </div>
            <div role="radiogroup" aria-label="Theme" class="grid grid-cols-2 gap-2">
                {#each THEMES as theme (theme.id)}
                    {@const selected = settings.theme === theme.id}
                    <button
                        type="button"
                        role="radio"
                        aria-checked={selected}
                        onclick={() => settings.setTheme(theme.id)}
                        class="flex flex-col gap-2 rounded-md border p-3 text-left transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50 {selected
                            ? 'border-brass bg-accent'
                            : ''}"
                    >
                        <span class="flex h-6 overflow-hidden rounded border">
                            {#each theme.swatch as colour}
                                <span class="flex-1" style="background: {colour}"></span>
                            {/each}
                        </span>
                        <span class="text-sm font-semibold">{theme.label}</span>
                        <span class="text-xs text-muted-foreground">{theme.description}</span>
                    </button>
                {/each}
            </div>
        </div>
    </section>
{/if}

{#if show("reduced-motion")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="reduced-motion" class="font-heading text-sm font-semibold tracking-wide">
                    Reduced motion
                </label>
                <p class="text-sm text-muted-foreground">
                    Turn off animations and transitions. "Follow system" uses your OS setting.
                </p>
            </div>
            <Select
                id="reduced-motion"
                value={settings.motion}
                onchange={(e) => settings.setMotion(resolveMotionPreference(e.currentTarget.value))}
            >
                <option value="system">Follow system</option>
                <option value="reduce">Reduce</option>
                <option value="full">Full motion</option>
            </Select>
        </div>
    </section>
{/if}

{#if show("accessible-font")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <label for="accessible-font" class="font-heading text-sm font-semibold tracking-wide">
                    Accessible font
                </label>
                <p class="text-sm text-muted-foreground">
                    Swap the game-style fonts for Atkinson Hyperlegible, designed for low vision and easier reading.
                </p>
            </div>
            <Switch
                id="accessible-font"
                checked={settings.accessibleFont}
                onCheckedChange={(v) => settings.setAccessibleFont(v)}
            />
        </div>
    </section>
{/if}
