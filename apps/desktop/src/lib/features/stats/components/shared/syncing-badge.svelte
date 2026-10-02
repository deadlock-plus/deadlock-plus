<script lang="ts">
    let { count = 1 }: { count?: number } = $props();
</script>

<span
    class="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground"
    title="Read from the game. The Deadlock API has not returned {count === 1 ? 'this match' : 'these matches'} yet."
>
    {count > 1 ? `${count} syncing` : "Syncing"}
</span>
