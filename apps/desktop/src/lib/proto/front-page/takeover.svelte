<script lang="ts">
    import type { Snippet } from "svelte";
    import { ExternalLink, ImageDown, X } from "@lucide/svelte";

    import Button from "@deadlock-plus/ui/button.svelte";

    let {
        width,
        height,
        angle = -1.4,
        tossAt = null,
        still = false,
        onopen,
        onclose,
        paper,
    }: {
        width: number;
        height: number;
        angle?: number;
        /** Freezes the toss this many ms in. */
        tossAt?: number | null;
        still?: boolean;
        onopen: () => void;
        onclose: () => void;
        paper: Snippet;
    } = $props();

    const TOSS_MS = 900;
    const RAIL = 200;
    const GAP = 28;
    const MARGIN = 28;

    let stageW = $state(0);
    let stageH = $state(0);
    const rad = $derived((Math.abs(angle) * Math.PI) / 180);
    const boxW = $derived(width * Math.cos(rad) + height * Math.sin(rad));
    const boxH = $derived(width * Math.sin(rad) + height * Math.cos(rad));
    const scale = $derived(
        stageW && stageH ? Math.min((stageW - RAIL - GAP - MARGIN * 2) / boxW, (stageH - MARGIN * 2) / boxH, 1.25) : 0,
    );

    const motion = $derived(
        still
            ? "animation: none;"
            : tossAt !== null
              ? `animation-delay: -${Math.min(tossAt, TOSS_MS)}ms; animation-play-state: paused;`
              : "",
    );

    function onkeydown(e: KeyboardEvent) {
        if (e.key === "Escape") onclose();
    }
</script>

<svelte:window {onkeydown} />

<div class="takeover" role="dialog" aria-modal="true" aria-label="Front page">
    <button type="button" class="table" aria-label="Close" onclick={onclose} style={motion}></button>
    <div class="stage" bind:clientWidth={stageW} bind:clientHeight={stageH}>
        <div class="spot" style="width: {width * scale}px; height: {height * scale}px;">
            <div class="toss" style="--angle: {angle}deg; {motion}">
                <div class="sheet" style="width: {width}px; height: {height}px; transform: scale({scale});">
                    {@render paper()}
                    <div class="fold" aria-hidden="true"></div>
                </div>
            </div>
        </div>
        <div class="rail" style="width: {RAIL}px;">
            <Button class="w-full justify-start" onclick={onopen}>
                <ExternalLink aria-hidden="true" />
                Open full match
            </Button>
            <Button variant="outline" class="w-full justify-start bg-background/60">
                <ImageDown aria-hidden="true" />
                Save image
            </Button>
            <Button variant="ghost" class="w-full justify-start" onclick={onclose}>
                <X aria-hidden="true" />
                Close
            </Button>
        </div>
    </div>
</div>

<style>
    .takeover {
        position: fixed;
        inset: 40px 0 0 0;
        z-index: 200;
    }

    .table {
        position: absolute;
        inset: 0;
        cursor: default;
        background:
            radial-gradient(ellipse at 50% 45%, transparent 30%, rgb(0 0 0 / 0.45) 100%),
            color-mix(in oklch, var(--background) 55%, rgb(0 0 0 / 0.7));
        backdrop-filter: blur(3px) saturate(0.7);
        animation: scrim 320ms ease-out both;
    }

    .stage {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 28px;
        padding: 0 28px;
        pointer-events: none;
    }

    .spot {
        position: relative;
        flex: none;
        pointer-events: auto;
    }

    .toss {
        position: absolute;
        inset: 0;
        transform: rotate(var(--angle));
        box-shadow:
            0 1px 2px rgb(0 0 0 / 0.35),
            0 14px 28px -6px rgb(0 0 0 / 0.5),
            0 38px 80px -18px rgb(0 0 0 / 0.6);
        animation: toss 900ms cubic-bezier(0.2, 0.7, 0.25, 1) both;
    }

    .sheet {
        position: absolute;
        left: 0;
        top: 0;
        transform-origin: 0 0;
        overflow: hidden;
        color: var(--ink);
        --paper: #ece4d1;
        --ink: #1d1a15;
        --ink-soft: #4b4337;
        --rule: #2b251d;
        --spot: #a3281c;
        --hk-ink: #8c5c06;
        --am-ink: #234a80;
        background-color: var(--paper);
        background-image:
            url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='240' height='240'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.85' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 .32 0 0 0 0 .26 0 0 0 0 .17 0 0 0 .13 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>"),
            radial-gradient(ellipse at 50% 40%, transparent 55%, rgb(150 112 50 / 0.16) 100%),
            linear-gradient(100deg, rgb(255 255 255 / 0.18), transparent 40%, rgb(120 90 40 / 0.06));
        box-shadow:
            inset 0 0 0 1px rgb(120 95 55 / 0.25),
            inset 0 0 40px rgb(140 105 50 / 0.18);
    }

    .fold {
        position: absolute;
        inset: 0;
        pointer-events: none;
        background: linear-gradient(
            to bottom,
            rgb(255 255 255 / 0.1) 0%,
            transparent 48.6%,
            rgb(70 50 20 / 0.12) 49.9%,
            rgb(255 255 255 / 0.32) 50.1%,
            transparent 51.6%,
            rgb(60 45 20 / 0.05) 100%
        );
        mix-blend-mode: multiply;
    }

    .rail {
        display: flex;
        flex-direction: column;
        gap: 8px;
        align-self: flex-end;
        margin-bottom: 40px;
        pointer-events: auto;
        animation: scrim 400ms 500ms ease-out both;
    }

    @keyframes scrim {
        from {
            opacity: 0;
        }
    }

    @keyframes toss {
        0% {
            transform: translate(55vw, 70vh) rotate(38deg) scale(1.35);
            opacity: 0;
        }
        12% {
            opacity: 1;
        }
        72% {
            transform: translate(0, 0) rotate(calc(var(--angle) - 2.2deg)) scale(1.03);
        }
        86% {
            transform: translate(0, -4px) rotate(calc(var(--angle) + 0.6deg)) scale(1);
        }
        100% {
            transform: rotate(var(--angle));
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .toss {
            animation-name: scrim;
        }
    }
</style>
