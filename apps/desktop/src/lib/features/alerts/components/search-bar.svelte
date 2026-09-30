<script lang="ts">
    import { Search, X } from "@lucide/svelte";

    type Props = {
        query: string;
        oninput: () => void;
        onclear: () => void;
    };

    let { query = $bindable(), oninput, onclear }: Props = $props();
</script>

<header class="flex flex-col gap-4">
    <div>
        <h1 class="text-3xl">Updates</h1>
        <p class="mt-1 text-base text-muted-foreground">Recent Deadlock patch notes and Steam announcements.</p>
    </div>
    <div class="relative w-full">
        <Search class="pointer-events-none absolute left-4 top-1/2 size-5 -translate-y-1/2 text-muted-foreground" />
        <input
            type="text"
            bind:value={query}
            {oninput}
            placeholder="Search patch notes... try a hero, an item, or describe the change"
            class="h-14 w-full rounded-xl border border-border bg-card pl-12 pr-12 text-lg outline-none focus-visible:ring-2 focus-visible:ring-brass/50"
        />
        {#if query}
            <button
                type="button"
                aria-label="Clear search"
                class="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                onclick={onclear}
            >
                <X class="size-5" />
            </button>
        {/if}
    </div>
</header>
