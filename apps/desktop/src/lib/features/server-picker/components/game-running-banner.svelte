<script lang="ts">
    import { TriangleAlert } from "@lucide/svelte";
</script>

<div
    role="alert"
    class="flex items-center gap-4 rounded-lg border-2 border-destructive/60 bg-destructive/10 px-5 py-4 text-destructive"
>
    <TriangleAlert class="size-8 shrink-0" aria-hidden="true" />
    <div>
        <p class="font-heading text-lg font-semibold">Deadlock is running</p>
        <p class="text-sm">
            Restart your game after changing anything on this page. Blocks only apply to connections made after the game
            starts.
        </p>
    </div>
</div>
