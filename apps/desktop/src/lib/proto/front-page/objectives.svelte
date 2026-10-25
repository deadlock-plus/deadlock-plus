<script lang="ts">
    import type { MatchTeam } from "@deadlock-plus/match/detail";
    import { clock, teamName, type ObjectiveMark } from "./fixture";

    let {
        marks,
        durationS,
        title = "Objectives",
        compact = false,
    }: { marks: ObjectiveMark[]; durationS: number; title?: string; compact?: boolean } = $props();

    const MARK_W = 22;
    const ROW_H = 15;
    let width = $state(0);

    function place(side: MatchTeam) {
        const last: number[] = [];
        return marks
            .filter((m) => m.by === side)
            .map((m) => {
                const x = (m.timeS / durationS) * width;
                let row = last.findIndex((l) => x - l >= MARK_W + 2);
                if (row < 0) row = last.length;
                last[row] = x;
                return { ...m, x, row };
            });
    }

    const top = $derived(width ? place("hidden-king") : []);
    const bottom = $derived(width ? place("archmother") : []);
    const rows = $derived(Math.max(1, ...top.map((m) => m.row + 1), ...bottom.map((m) => m.row + 1)));
    const ticks = $derived(
        Array.from({ length: Math.floor(durationS / 300) + 1 }, (_, i) => i * 300).filter(
            (tk) => durationS - tk >= 150,
        ),
    );
    const kinds = $derived([...new Map(marks.map((m) => [m.short, m.label])).entries()]);
</script>

<section class="obj" class:compact>
    <div class="label">
        <h4>{title}</h4>
        <p><i class="hidden-king"></i>{teamName("hidden-king")}</p>
        <p><i class="archmother"></i>{teamName("archmother")}</p>
    </div>
    <div class="track" bind:clientWidth={width} style="--rows: {rows}; --row: {ROW_H}px; --mark: {MARK_W}px;">
        <div class="side up">
            {#each top as m (m.timeS + m.short)}
                <span
                    class="mark hidden-king"
                    style="left: {m.x}px; bottom: {m.row * ROW_H}px;"
                    title="{m.label} {clock(m.timeS)}">{m.short}</span
                >
            {/each}
        </div>
        <div class="axis">
            {#each ticks as tk (tk)}
                <span style="left: {(tk / durationS) * 100}%">{clock(tk)}</span>
            {/each}
            <span class="end" style="left: 100%">{clock(durationS)}</span>
        </div>
        <div class="side down">
            {#each bottom as m (m.timeS + m.short)}
                <span
                    class="mark archmother"
                    style="left: {m.x}px; top: {m.row * ROW_H}px;"
                    title="{m.label} {clock(m.timeS)}">{m.short}</span
                >
            {/each}
        </div>
    </div>
    <ul class="key">
        {#each kinds as [short, label] (short)}
            <li><b>{short}</b> {label}</li>
        {/each}
    </ul>
</section>

<style>
    .obj {
        display: grid;
        grid-template-columns: 128px 1fr 190px;
        gap: 14px;
        align-items: center;
        margin-top: 8px;
        padding-top: 6px;
        border-top: 3px double var(--rule);
        font-family: var(--obj-text, "Radiance", sans-serif);
        font-size: 10.5px;
    }

    h4 {
        margin: 0 0 3px;
        font-family: var(--obj-head, "Valve Pulp", sans-serif);
        font-size: 13px;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--ink);
    }

    .label p {
        margin: 0;
        display: flex;
        align-items: center;
        gap: 5px;
        white-space: nowrap;
    }

    i {
        width: 9px;
        height: 9px;
        flex: none;
    }

    i.hidden-king,
    .mark.hidden-king {
        background: var(--hk-ink);
    }

    i.archmother,
    .mark.archmother {
        background: var(--am-ink);
    }

    .track {
        position: relative;
        display: grid;
        grid-template-rows: calc(var(--rows) * var(--row)) 16px calc(var(--rows) * var(--row));
    }

    .side {
        position: relative;
    }

    .mark {
        position: absolute;
        width: var(--mark);
        height: calc(var(--row) - 2px);
        transform: translateX(-50%);
        display: grid;
        place-items: center;
        font-weight: 700;
        font-size: 9px;
        letter-spacing: 0.02em;
        color: var(--paper);
    }

    .axis {
        position: relative;
        border-top: 1px solid var(--rule);
        border-bottom: 1px solid var(--rule);
    }

    .axis span {
        position: absolute;
        top: 1px;
        transform: translateX(-50%);
        font-size: 9.5px;
        color: var(--ink-soft);
        font-variant-numeric: tabular-nums;
    }

    .axis span::before {
        content: "";
        position: absolute;
        left: 50%;
        top: -4px;
        height: 3px;
        border-left: 1px solid var(--rule);
    }

    .axis span:first-child {
        transform: none;
    }

    .axis span.end {
        transform: translateX(-100%);
        font-weight: 700;
        color: var(--ink);
    }

    .key {
        margin: 0;
        padding: 0;
        list-style: none;
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0 8px;
        font-size: 9.5px;
        line-height: 1.25;
        color: var(--ink-soft);
    }

    .key b {
        display: inline-block;
        min-width: 16px;
        color: var(--ink);
    }

    .compact {
        grid-template-columns: 92px 1fr;
    }

    .compact .key {
        display: none;
    }
</style>
