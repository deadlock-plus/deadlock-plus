<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { Copy } from "@lucide/svelte";

    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";

    import { frameLayer } from "$lib/features/performance/layer.svelte";

    const status = $derived(frameLayer.status);

    async function copyLaunchOption() {
        if (!status) return;
        try {
            await navigator.clipboard.writeText(status.launchOption);
            toast.success("Copied the launch option");
        } catch {
            toast.error("Couldn't copy the launch option");
        }
    }

    onMount(() => void frameLayer.load());
</script>

{#if status?.supported}
    <Card radius="md" padding="row" class="flex flex-col gap-3">
        <div class="flex items-start justify-between gap-4">
            <div class="flex flex-col gap-1">
                <p class="text-sm font-medium">
                    {status.installed ? "Frame layer installed" : "Frame layer not installed"}
                </p>
                <p class="text-xs text-muted-foreground">
                    {#if status.installed}
                        It only loads into games started with the launch option below.
                    {:else if status.libraryAvailable}
                        Installs a small Vulkan layer into your home folder. It does nothing until you add the launch
                        option.
                    {:else}
                        This build of Deadlock+ does not include the layer, so it cannot be installed.
                    {/if}
                </p>
            </div>
            {#if status.installed}
                <Button variant="outline" size="sm" disabled={frameLayer.busy} onclick={() => frameLayer.uninstall()}>
                    Remove
                </Button>
            {:else}
                <Button
                    size="sm"
                    disabled={frameLayer.busy || !status.libraryAvailable}
                    onclick={() => frameLayer.install()}
                >
                    Install
                </Button>
            {/if}
        </div>
        <div class="flex flex-col gap-1">
            <p class="text-xs text-muted-foreground">
                Steam, Deadlock, Properties, Launch options. Then start the game and press Start here.
            </p>
            <div class="flex items-center gap-2">
                <code class="flex-1 rounded-md border border-border bg-muted px-3 py-2 font-mono text-xs">
                    {status.launchOption}
                </code>
                <Button variant="outline" size="sm" onclick={copyLaunchOption}><Copy /> Copy</Button>
            </div>
        </div>
        {#if frameLayer.error}<p class="text-sm text-destructive">{frameLayer.error}</p>{/if}
    </Card>
{/if}
