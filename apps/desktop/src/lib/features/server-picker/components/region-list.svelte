<script lang="ts">
    import EmptyState from "$lib/ui/empty-state.svelte";
    import type { ServerPicker } from "../picker.svelte";
    import ServerRow from "./server-row.svelte";

    let { picker }: { picker: ServerPicker } = $props();
</script>

<div class="flex flex-col gap-2">
    {#each picker.visibleRegions as group (group.id)}
        {@const members = picker.members(group)}
        {@const expanded = group.isCluster && picker.isExpanded(group)}
        <ServerRow
            {group}
            expandable={group.isCluster && members.length > 0}
            {expanded}
            memberCount={members.length}
            blockedMembers={members.filter((m) => picker.blockedIds.has(m.id)).length}
            blocked={picker.blockedIds.has(group.id)}
            busy={picker.busyIds.has(group.id)}
            external={picker.externalIds.has(group.id)}
            externalLabel={picker.externalLabel}
            pending={picker.pinging}
            ping={picker.pingOf(group)}
            allSiblingsBlocked={picker.allSiblingsBlocked(group)}
            siblingNames={picker.siblingNames(group)}
            onToggle={(checked) => picker.toggleGroup(group, checked)}
            onExpand={() => picker.toggleExpanded(group.id)}
            onBlockSiblings={() => picker.blockSiblings(group)}
        />

        {#if expanded}
            <div class="ml-6 flex flex-col gap-1.5 border-l border-border pl-3">
                {#each picker.visibleMembers(group) as member (member.id)}
                    <ServerRow
                        group={member}
                        nested
                        blocked={picker.blockedIds.has(member.id) || picker.blockedIds.has(group.id)}
                        lockedBy={picker.blockedIds.has(group.id) ? group.description : null}
                        busy={picker.busyIds.has(member.id)}
                        external={false}
                        externalLabel={picker.externalLabel}
                        pending={picker.pinging}
                        ping={picker.pings[member.id]}
                        onToggle={(checked) => picker.toggleGroup(member, checked)}
                    />
                {/each}
            </div>
        {/if}
    {:else}
        <EmptyState>No regions match "{picker.search}".</EmptyState>
    {/each}
</div>
