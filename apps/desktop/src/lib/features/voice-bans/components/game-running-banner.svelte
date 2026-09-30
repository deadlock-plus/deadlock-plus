<script lang="ts">
    import { TriangleAlert } from "@lucide/svelte";
</script>

<div
    class="flex items-center gap-4 rounded-lg border-2 border-destructive/60 bg-destructive/10 px-5 py-4 text-destructive"
>
    <TriangleAlert class="size-8 shrink-0" />
    <div>
        <p class="font-heading text-lg font-semibold">Deadlock is running</p>
        <p class="text-sm">
            Mutes can't be changed while the game is open. It rewrites the file when it closes and would erase your
            changes. Close Deadlock to edit this list.
        </p>
    </div>
</div>
