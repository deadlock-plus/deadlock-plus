<script lang="ts">
    import { toast } from "svelte-sonner";

    import Button from "$lib/components/ui/button.svelte";
    import Input from "$lib/components/ui/input.svelte";
    import Switch from "$lib/components/ui/switch.svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import {
        cleanupMatches,
        formatBytes,
        gbToMb,
        listCleanupRules,
        matchesTotal,
        mbToGb,
        ruleLabel,
        saveCleanupRules,
        withAllRules,
        type CleanupMatch,
        type CleanupRule,
    } from "$lib/features/demos/demos";

    let { open = $bindable(false), onreview }: { open?: boolean; onreview: (fileNames: string[]) => void } = $props();

    const SAVE_DELAY_MS = 400;

    let rules = $state<CleanupRule[]>([]);
    let matches = $state<CleanupMatch[]>([]);
    let loading = $state(false);
    let timer: ReturnType<typeof setTimeout> | undefined;
    let generation = 0;

    const total = $derived(matchesTotal(matches));
    const anyEnabled = $derived(rules.some((r) => r.enabled));

    $effect(() => {
        if (open) void load();
        else clearTimeout(timer);
    });

    async function load() {
        loading = true;
        try {
            rules = withAllRules(await listCleanupRules());
            await refresh();
        } catch (e) {
            toast.error(String(e));
        } finally {
            loading = false;
        }
    }

    async function refresh() {
        const mine = ++generation;
        const found = await cleanupMatches();
        if (mine === generation) matches = found;
    }

    function changed() {
        clearTimeout(timer);
        timer = setTimeout(async () => {
            try {
                await saveCleanupRules($state.snapshot(rules));
                await refresh();
            } catch (e) {
                toast.error(String(e));
            }
        }, SAVE_DELAY_MS);
    }

    function setWhole(rule: CleanupRule, raw: string) {
        const value = Math.floor(Number(raw));
        if (rule.kind === "olderThanDays" && Number.isFinite(value) && value >= 1) {
            rule.days = value;
            changed();
        }
    }

    function setGb(rule: CleanupRule, raw: string) {
        if (rule.kind === "largerThanMb" && raw.trim() !== "" && Number(raw) > 0) {
            rule.mb = gbToMb(Number(raw));
            changed();
        }
    }

    function labelsFor(m: CleanupMatch) {
        return m.ruleIds
            .map((id) => rules.find((r) => r.id === id))
            .filter((r): r is CleanupRule => r !== undefined)
            .map(ruleLabel)
            .join(", ");
    }

    function review() {
        open = false;
        onreview(matches.map((m) => m.fileName));
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content>
        <div class="flex flex-col gap-1.5">
            <Dialog.Title>Clean up replays</Dialog.Title>
            <Dialog.Description>
                Pick what counts as clutter. Nothing is deleted until you review the list and confirm. Pinned replays
                are always skipped.
            </Dialog.Description>
        </div>

        <ul class="flex flex-col gap-2">
            {#each rules as rule (rule.id)}
                <li class="flex items-center gap-3 rounded-md border border-border px-3 py-2">
                    <Switch
                        checked={rule.enabled}
                        aria-label={ruleLabel(rule)}
                        onCheckedChange={(on) => {
                            rule.enabled = on;
                            changed();
                        }}
                    />
                    {#if rule.kind === "olderThanDays"}
                        <span class="text-sm">Older than</span>
                        <Input
                            type="number"
                            min="1"
                            step="1"
                            class="h-8 w-20"
                            aria-label="Days"
                            value={String(rule.days)}
                            oninput={(e) => setWhole(rule, e.currentTarget.value)}
                        />
                        <span class="text-sm">days</span>
                    {:else if rule.kind === "largerThanMb"}
                        <span class="text-sm">Larger than</span>
                        <Input
                            type="number"
                            min="0.1"
                            step="0.5"
                            class="h-8 w-20"
                            aria-label="Gigabytes"
                            value={String(mbToGb(rule.mb))}
                            oninput={(e) => setGb(rule, e.currentTarget.value)}
                        />
                        <span class="text-sm">GB</span>
                    {:else}
                        <span class="text-sm">{ruleLabel(rule)}</span>
                    {/if}
                </li>
            {/each}
        </ul>

        <div class="flex min-h-0 flex-1 flex-col gap-2">
            <p class="text-sm">
                {#if !anyEnabled}
                    <span class="text-muted-foreground">Turn on a rule to see what it would remove.</span>
                {:else if loading}
                    <span class="text-muted-foreground">Checking replays…</span>
                {:else}
                    {total.count} replay{total.count === 1 ? "" : "s"} match, {formatBytes(total.bytes)}
                {/if}
            </p>
            {#if anyEnabled && matches.length > 0}
                <ul class="max-h-40 overflow-y-auto rounded-md border border-border text-xs">
                    {#each matches as m (m.fileName)}
                        <li
                            class="flex items-baseline justify-between gap-3 border-b border-border px-3 py-1.5 last:border-b-0"
                        >
                            <span>
                                Match {m.matchId}
                                <span class="text-muted-foreground"> · {labelsFor(m)}</span>
                            </span>
                            <span class="shrink-0 text-muted-foreground">{formatBytes(m.size)}</span>
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>

        <div class="flex justify-end gap-2">
            <Button variant="outline" onclick={() => (open = false)}>Close</Button>
            <Button disabled={!anyEnabled || loading || matches.length === 0} onclick={review}>
                Review {total.count} replay{total.count === 1 ? "" : "s"}
            </Button>
        </div>
    </Dialog.Content>
</Dialog.Root>
