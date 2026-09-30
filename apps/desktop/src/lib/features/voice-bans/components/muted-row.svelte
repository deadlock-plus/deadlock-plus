<script lang="ts">
    import { ExternalLink } from "@lucide/svelte";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import IconButton from "$lib/ui/icon-button.svelte";
    import type { Profile } from "../profiles";

    let {
        id,
        profile,
        selected,
        disabled,
        ontoggle,
        onopen,
        onunmute,
    }: {
        id: string;
        profile: Profile | undefined;
        selected: boolean;
        disabled: boolean;
        ontoggle: (on: boolean) => void;
        onopen: () => void;
        onunmute: () => void;
    } = $props();
</script>

<Card as="li" radius="md" padding="none" class="flex items-center gap-3 px-4 py-2">
    <input
        type="checkbox"
        aria-label="Select {profile?.name ?? id}"
        checked={selected}
        onchange={(e) => ontoggle(e.currentTarget.checked)}
    />
    {#if profile?.avatar}
        <img src={profile.avatar} alt="" class="size-8 shrink-0 rounded-sm" />
    {:else}
        <div class="size-8 shrink-0 rounded-sm bg-accent"></div>
    {/if}
    <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-medium">{profile?.name ?? "Unknown player"}</p>
        <p class="text-xs text-muted-foreground">{id}</p>
    </div>
    <IconButton size="sm" label="Open Statlocker profile" title="Statlocker profile" onclick={onopen}>
        <ExternalLink />
    </IconButton>
    <Button size="sm" variant="outline" {disabled} onclick={onunmute}>Unmute</Button>
</Card>
