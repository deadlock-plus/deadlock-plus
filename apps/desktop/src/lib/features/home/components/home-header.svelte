<script lang="ts">
    import { User } from "@lucide/svelte";

    type Props = {
        avatar: string | null;
        greeting: string;
    };

    let { avatar, greeting }: Props = $props();
</script>

<header class="flex flex-col items-center gap-4 text-center">
    {#if avatar}
        <img src={avatar} alt="" class="size-28 shrink-0 rounded-xl border border-border" />
    {:else}
        <div class="flex size-28 shrink-0 items-center justify-center rounded-xl border border-border bg-card">
            <User class="size-14 text-muted-foreground" />
        </div>
    {/if}
    <div class="min-w-0 max-w-full">
        <h1 class="text-4xl leading-tight lg:text-5xl">{greeting}</h1>
    </div>
</header>
