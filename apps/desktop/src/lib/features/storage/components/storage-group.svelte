<script lang="ts">
    import { formatBytes } from "$lib/features/demos/demos";
    import { ownerText, type EntryId, type Group, type StatsById } from "$lib/features/storage/storage";
    import StorageEntry from "./storage-entry.svelte";

    type Props = {
        group: Group;
        stats: StatsById;
        failed: Set<EntryId>;
        total: number;
        now: number;
        showPaths: boolean;
        clearing: boolean;
        onretry: (id: EntryId) => void;
        onopen: (link: string) => void;
        onreveal: (id: EntryId) => void;
        onclear: (id: EntryId) => void;
    };

    let { group, stats, failed, total, now, showPaths, clearing, onretry, onopen, onreveal, onclear }: Props = $props();

    const owner = $derived(ownerText(group.owner.id));
</script>

<section class="flex flex-col gap-1.5">
    <div class="flex items-baseline justify-between gap-4 px-1">
        <div class="flex items-baseline gap-2">
            <h2 class="text-lg">{owner.label}</h2>
            <p class="text-xs text-muted-foreground">{owner.blurb}</p>
        </div>
        <p class="text-sm tabular-nums text-muted-foreground">{formatBytes(group.bytes)}</p>
    </div>

    <ul class="flex flex-col gap-1.5">
        {#each group.entries as entry (entry.id)}
            <StorageEntry
                {entry}
                stats={stats[entry.id]}
                failed={failed.has(entry.id)}
                {total}
                {now}
                showPath={showPaths}
                {clearing}
                onretry={() => onretry(entry.id)}
                {onopen}
                onreveal={() => onreveal(entry.id)}
                onclear={() => onclear(entry.id)}
            />
        {/each}
    </ul>
</section>
