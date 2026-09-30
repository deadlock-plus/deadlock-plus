<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { saveTextFile } from "$lib/core/files";
    import { Copy, FolderOpen, RefreshCw, Save } from "@lucide/svelte";
    import Button from "$lib/ui/button.svelte";
    import Input from "$lib/ui/input.svelte";
    import Select from "$lib/ui/select.svelte";
    import Switch from "$lib/ui/switch.svelte";
    import {
        exportLogs,
        filterEntries,
        LEVELS,
        openLogDir,
        readLogs,
        type LogEntry,
        type LogLevel,
    } from "$lib/features/logging/logs";

    let { full = false }: { full?: boolean } = $props();

    const COMPACT_ROWS = 300;
    const TAG_CLASS: Record<LogLevel, string> = {
        ERROR: "text-red-400",
        WARN: "text-yellow-400",
        INFO: "text-green-400",
        DEBUG: "text-green-400",
        TRACE: "text-blue-400",
    };

    let entries = $state<LogEntry[] | null>(null);
    let failed = $state<string | null>(null);
    let level = $state<LogLevel>("INFO");
    let query = $state("");

    const visible = $derived(entries ? filterEntries(entries, level, query) : []);
    const shown = $derived(full ? visible : visible.slice(-COMPACT_ROWS));

    let autoScroll = $state(true);
    let scroller = $state<HTMLDivElement | null>(null);

    $effect(() => {
        shown;
        if (autoScroll && scroller) scroller.scrollTop = scroller.scrollHeight;
    });

    async function load() {
        try {
            entries = await readLogs();
            failed = null;
        } catch (e) {
            failed = e instanceof Error ? e.message : String(e);
        }
    }

    onMount(load);

    async function copyLogs() {
        try {
            await navigator.clipboard.writeText(await exportLogs());
            toast.success("Copied the log");
        } catch {
            toast.error("Couldn't copy the log");
        }
    }

    async function saveLogs() {
        try {
            const saved = await saveTextFile(
                { defaultName: "deadlock-plus-log.txt", filterName: "Text", extension: "txt" },
                await exportLogs(),
            );
            if (saved) toast.success("Saved the log");
        } catch (e) {
            toast.error(`Couldn't save the log: ${e instanceof Error ? e.message : e}`);
        }
    }

    async function openFolder() {
        try {
            await openLogDir();
        } catch (e) {
            toast.error(`Couldn't open the log folder: ${e instanceof Error ? e.message : e}`);
        }
    }
</script>

<div class="flex min-h-0 flex-col gap-3 {full ? 'h-full' : ''}">
    <div class="flex flex-wrap items-center gap-2">
        <Select
            value={level}
            onchange={(e) => (level = e.currentTarget.value as LogLevel)}
            aria-label="Minimum level"
            class="w-28"
        >
            {#each LEVELS as l (l)}
                <option value={l}>{l}</option>
            {/each}
        </Select>
        <Input bind:value={query} placeholder="Search" class="min-w-32 flex-1" aria-label="Search the log" />
        <label class="flex items-center gap-2 text-xs text-muted-foreground">
            <Switch bind:checked={autoScroll} aria-label="Auto-scroll" />
            Auto-scroll
        </label>
        <Button variant="outline" size="sm" onclick={load} aria-label="Refresh"><RefreshCw /></Button>
        <Button variant="outline" size="sm" onclick={copyLogs}><Copy />Copy</Button>
        <Button variant="outline" size="sm" onclick={saveLogs}><Save />Save</Button>
        <Button variant="outline" size="sm" onclick={openFolder}><FolderOpen />Open folder</Button>
    </div>

    {#if failed}
        <p class="text-xs text-destructive">Couldn't read the log: {failed}</p>
    {:else if !entries}
        <p class="text-xs text-muted-foreground">Loading...</p>
    {:else if shown.length === 0}
        <p class="text-xs text-muted-foreground">Nothing matches.</p>
    {:else}
        <div
            bind:this={scroller}
            class="{full
                ? 'min-h-0 flex-1'
                : 'max-h-80'} overflow-auto rounded bg-muted/40 p-2 font-mono text-[11px] leading-snug select-text"
        >
            {#if visible.length > shown.length}
                <p class="mb-1 text-muted-foreground">Showing the latest {shown.length} of {visible.length}.</p>
            {/if}
            {#each shown as e, i (i)}
                <div class="whitespace-pre-wrap break-words">
                    <span class="text-blue-400">[{e.time}]</span>
                    <span class={TAG_CLASS[e.level]}>[{e.thread} | {e.level}]</span>
                    <span class="text-cyan-400">[{e.logger}]</span>:
                    <span class={e.level === "ERROR" ? "text-red-400" : ""}>{e.message}</span>
                </div>
            {/each}
        </div>
    {/if}
</div>
