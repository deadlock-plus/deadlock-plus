<script lang="ts">
    import Card from "$lib/ui/card.svelte";
    import SettingRow from "$lib/ui/setting-row.svelte";
    import Button from "$lib/ui/button.svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { THEMES, resolveMotionPreference } from "$lib/features/settings/themes";

    let { show }: { show: (id: string) => boolean } = $props();
</script>

{#if show("theme")}
    <Card as="section">
        <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1">
                <h3 class="font-heading text-sm font-semibold tracking-wide">Theme</h3>
                <p class="text-sm text-muted-foreground">Pick the colours the app uses.</p>
            </div>
            <div role="radiogroup" aria-label="Theme" class="grid grid-cols-2 gap-2">
                {#each THEMES as theme (theme.id)}
                    {@const selected = settings.theme === theme.id}
                    <Button
                        variant="unstyled"
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
                    </Button>
                {/each}
            </div>
        </div>
    </Card>
{/if}

{#if show("reduced-motion")}
    <Card as="section">
        <SettingRow
            label="Reduced motion"
            for="reduced-motion"
            description={'Turn off animations and transitions. "Follow system" uses your OS setting.'}
        >
            <Select
                id="reduced-motion"
                value={settings.motion}
                onchange={(e) => settings.setMotion(resolveMotionPreference(e.currentTarget.value))}
            >
                <option value="system">Follow system</option>
                <option value="reduce">Reduce</option>
                <option value="full">Full motion</option>
            </Select>
        </SettingRow>
    </Card>
{/if}

{#if show("accessible-font")}
    <Card as="section">
        <SettingRow
            label="Accessible font"
            for="accessible-font"
            description="Swap the game-style fonts for Atkinson Hyperlegible, designed for low vision and easier reading."
        >
            <Switch
                id="accessible-font"
                checked={settings.accessibleFont}
                onCheckedChange={(v) => settings.setAccessibleFont(v)}
            />
        </SettingRow>
    </Card>
{/if}
