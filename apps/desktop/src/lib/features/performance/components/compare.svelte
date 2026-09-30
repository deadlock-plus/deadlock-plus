<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { Copy, Save, Trash2 } from "@lucide/svelte";

    import { saveTextFile } from "$lib/core/files";

    import Badge from "$lib/ui/badge.svelte";
    import Button from "$lib/ui/button.svelte";

    import { formatDuration } from "$lib/features/performance/performance";
    import { addonDiff, compareRuns, comparisonReport, formatValue, type Better } from "$lib/features/performance/runs";
    import { savedRuns } from "$lib/features/performance/runs.svelte";

    let aId = $state("");
    let bId = $state("");

    const runs = $derived(savedRuns.runs);
    const a = $derived(runs.find((r) => r.id === aId));
    const b = $derived(runs.find((r) => r.id === bId));
    const rows = $derived(a && b ? compareRuns(a, b) : []);
    const diff = $derived(a && b ? addonDiff(a, b) : { onlyInA: [], onlyInB: [] });
    const onlyInA = $derived(diff.onlyInA);
    const onlyInB = $derived(diff.onlyInB);
    const wins = $derived({
        a: rows.filter((r) => r.better === "a").length,
        b: rows.filter((r) => r.better === "b").length,
    });

    onMount(() => void savedRuns.load());

    async function copyReport() {
        if (!a || !b) return;
        try {
            await navigator.clipboard.writeText(comparisonReport(a, b));
            toast.success("Copied the comparison");
        } catch {
            toast.error("Couldn't copy the comparison");
        }
    }

    async function saveReport() {
        if (!a || !b) return;
        try {
            const saved = await saveTextFile(
                { defaultName: "deadlock-plus-comparison.txt", filterName: "Text", extension: "txt" },
                comparisonReport(a, b),
            );
            if (saved) toast.success("Saved the comparison");
        } catch (e) {
            toast.error(`Couldn't save the comparison: ${e instanceof Error ? e.message : e}`);
        }
    }

    $effect(() => {
        if (runs.length >= 2 && !a && !b) {
            aId = runs[1].id;
            bId = runs[0].id;
        }
        if (aId && !a) aId = "";
        if (bId && !b) bId = "";
    });

    const fmt = formatValue;
    const cell = (better: Better, side: "a" | "b") =>
        better === side ? "font-semibold text-success" : "text-foreground";
    const date = (t: number) => new Date(t).toLocaleString();
</script>

{#snippet picker(id: string, onchange: (v: string) => void, label: string)}
    <label class="flex min-w-0 flex-1 flex-col gap-1 text-xs text-muted-foreground">
        {label}
        <select
            class="h-9 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground"
            value={id}
            onchange={(e) => onchange(e.currentTarget.value)}
        >
            <option value="">Pick a run</option>
            {#each runs as r (r.id)}
                <option value={r.id}>{r.label} · {date(r.savedAt)}</option>
            {/each}
        </select>
    </label>
{/snippet}

<div class="flex flex-col gap-4">
    <p class="text-sm text-muted-foreground">
        Compare two saved runs side by side. Play the same map for a similar length of time in both, once with a mod on
        and once with it off. Frame pacing varies between matches, so a difference is consistent with the mod being the
        cause, not proof of it.
    </p>

    {#if runs.length < 2}
        <div class="rounded-md border border-border bg-card px-4 py-6 text-center text-sm text-muted-foreground">
            {runs.length === 0 ? "No saved runs yet." : "One saved run so far."} Record a run on the Frametimes tab and save
            it with a label. You need two to compare.
        </div>
    {:else}
        <div class="flex gap-3">
            {@render picker(aId, (v) => (aId = v), "Run A")}
            {@render picker(bId, (v) => (bId = v), "Run B")}
        </div>

        {#if a && b}
            {#if a.id === b.id}
                <p class="text-sm text-warning">Pick two different runs.</p>
            {:else}
                <div class="overflow-hidden rounded-md border border-border bg-card">
                    <table class="w-full text-sm">
                        <thead class="text-left text-xs text-muted-foreground">
                            <tr class="border-b border-border">
                                <th class="px-4 py-2 font-normal"></th>
                                <th class="px-4 py-2 font-normal">{a.label}</th>
                                <th class="px-4 py-2 font-normal">{b.label}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each rows as row (row.key)}
                                <tr class="border-b border-border last:border-0">
                                    <td class="px-4 py-2 text-muted-foreground">{row.label}</td>
                                    <td class="px-4 py-2 font-mono {cell(row.better, 'a')}">{fmt(row.a, row.unit)}</td>
                                    <td class="px-4 py-2 font-mono {cell(row.better, 'b')}">{fmt(row.b, row.unit)}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
                <div class="flex gap-2">
                    <Button variant="outline" size="sm" onclick={copyReport}><Copy /> Copy report</Button>
                    <Button variant="outline" size="sm" onclick={saveReport}><Save /> Save report</Button>
                </div>
                <p class="text-xs text-muted-foreground">
                    Green marks the better side. Differences under 3% count as a tie. Run A was {formatDuration(
                        a.stats.durationMs,
                    )} long, run B {formatDuration(b.stats.durationMs)}.
                    {#if wins.a === 0 && wins.b === 0}
                        No clear difference.
                    {:else if wins.a > 0 && wins.b > 0}
                        Mixed result: {wins.a} measures favour A, {wins.b} favour B.
                    {:else}
                        Every measure that differs favours {wins.a > 0 ? "A" : "B"}.
                    {/if}
                </p>
                {#if onlyInA.length > 0 || onlyInB.length > 0}
                    <div class="flex flex-col gap-2 rounded-md border border-border bg-card px-4 py-3 text-sm">
                        <p class="text-xs text-muted-foreground">Addons that differ between the runs</p>
                        {#each [{ name: a.label, list: onlyInA }, { name: b.label, list: onlyInB }] as side}
                            {#if side.list.length > 0}
                                <div class="flex flex-wrap items-center gap-1.5">
                                    <span class="text-muted-foreground">Only on in {side.name}:</span>
                                    {#each side.list as name}<Badge variant="outline">{name}</Badge>{/each}
                                </div>
                            {/if}
                        {/each}
                    </div>
                {:else}
                    <p class="text-xs text-warning">
                        Both runs had the same addons on, so any difference comes from something else.
                    </p>
                {/if}
            {/if}
        {/if}

        <ul class="flex flex-col divide-y divide-border rounded-md border border-border bg-card text-sm">
            {#each runs as r (r.id)}
                <li class="flex items-center justify-between gap-3 px-4 py-2">
                    <div class="min-w-0">
                        <p class="truncate">{r.label}</p>
                        <p class="text-xs text-muted-foreground">
                            {date(r.savedAt)} · {formatDuration(r.stats.durationMs)} · {r.addons.length} addon{r.addons
                                .length === 1
                                ? ""
                                : "s"} on
                        </p>
                    </div>
                    <Button
                        variant="ghost"
                        size="sm"
                        aria-label="Delete run {r.label}"
                        onclick={() => savedRuns.remove(r.id)}
                    >
                        <Trash2 />
                    </Button>
                </li>
            {/each}
        </ul>
        {#if savedRuns.error}<p class="text-sm text-destructive">{savedRuns.error}</p>{/if}
    {/if}
</div>
