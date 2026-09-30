<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { Copy, Download, RefreshCw } from "@lucide/svelte";
    import Button from "$lib/ui/button.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { updater } from "$lib/features/updates/updater.svelte";
    import { updateLine } from "$lib/features/updates/status";
    import ReleaseNotes from "$lib/features/updates/components/release-notes.svelte";
    import { whatsNew } from "$lib/features/updates/whats-new.svelte";
    import { aboutGroups, aboutText, getAppInfo, type AppInfo } from "$lib/features/settings/about";

    let { show }: { show: (id: string) => boolean } = $props();

    let info = $state<AppInfo | null>(null);

    onMount(() => {
        getAppInfo()
            .then((i) => (info = i))
            .catch(() => {});
    });

    async function copyInfo() {
        if (!info) return;
        try {
            await navigator.clipboard.writeText(aboutText(info));
            toast.success("Copied version details");
        } catch {
            toast.error("Couldn't copy to the clipboard");
        }
    }
</script>

{#if show("version") && info}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <h2 class="font-heading text-sm font-semibold tracking-wide">Version</h2>
            <Button variant="outline" size="sm" onclick={copyInfo}>
                <Copy />
                Copy
            </Button>
        </div>
        <div class="mt-3 flex flex-col gap-4">
            {#each aboutGroups(info) as group (group.title)}
                <div>
                    <h3 class="mb-1 text-xs uppercase tracking-widest text-muted-foreground/70">{group.title}</h3>
                    <dl class="grid grid-cols-[8.5rem_1fr] gap-x-4 gap-y-1 text-sm">
                        {#each group.rows as [label, value] (label)}
                            <dt class="text-muted-foreground">{label}</dt>
                            <dd class="min-w-0 break-words select-text">{value}</dd>
                        {/each}
                    </dl>
                </div>
            {/each}
        </div>
    </section>
{/if}

{#if show("updates")}
    <section class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-1">
                <h2 class="font-heading text-sm font-semibold tracking-wide">App updates</h2>
                <p class="text-sm text-muted-foreground" aria-live="polite">{updateLine(updater)}</p>
            </div>
            {#if updater.phase === "available"}
                <Button size="sm" onclick={() => updater.install()}>
                    <Download />
                    Install {updater.version}
                </Button>
            {:else}
                <Button
                    variant="outline"
                    size="sm"
                    disabled={updater.phase === "checking" || updater.phase === "downloading"}
                    onclick={() => updater.check()}
                >
                    <RefreshCw />
                    Check now
                </Button>
            {/if}
        </div>
        <div class="mt-4 flex items-center justify-between gap-4 border-t pt-4">
            <div class="flex flex-col gap-1">
                <label for="auto-update-check" class="font-heading text-sm font-semibold tracking-wide"
                    >Check automatically</label
                >
                <p class="text-sm text-muted-foreground">
                    Asks GitHub for a newer release on launch and every hour after. Nothing installs without your click.
                </p>
            </div>
            <Switch
                id="auto-update-check"
                checked={settings.autoUpdateCheck}
                onCheckedChange={(v) => settings.setAutoUpdateCheck(v)}
            />
        </div>
    </section>
{/if}

{#if show("whats-new")}
    <section class="rounded-lg border bg-card p-4">
        <h2 class="font-heading text-sm font-semibold tracking-wide">What's new</h2>
        <div class="mt-3">
            {#if whatsNew.history.length > 0}
                <ReleaseNotes entries={whatsNew.history} />
            {:else}
                <p class="text-sm text-muted-foreground">No release notes yet.</p>
            {/if}
        </div>
    </section>
{/if}
