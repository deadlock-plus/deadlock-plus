<script lang="ts">
    import { TriangleAlert } from "@lucide/svelte";
    import { platform, platformName } from "$lib/core/platform";
</script>

<div class="flex items-center gap-2 rounded-md border border-warning/40 bg-warning/10 px-3 py-2 text-sm text-warning">
    <TriangleAlert class="size-4 shrink-0" />
    Blocking servers isn't available on {platformName(platform)} yet.
</div>
