<script lang="ts">
    import type { Snippet } from "svelte";
    import { overlayHost } from "./overlay-host.svelte";

    let { children }: { children?: Snippet } = $props();
</script>

<div class="relative flex min-w-0 flex-1">
    <main
        inert={overlayHost.count > 0}
        class="noir-panel min-w-0 flex-1 overflow-y-auto rounded-l-xl border-y border-l border-border"
    >
        {@render children?.()}
    </main>
    <div
        bind:this={overlayHost.el}
        class="pointer-events-none absolute inset-0 isolate z-(--z-content-overlay) overflow-hidden rounded-l-xl"
    ></div>
</div>
