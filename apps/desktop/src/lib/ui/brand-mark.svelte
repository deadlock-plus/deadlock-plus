<script lang="ts">
    import { cn } from "$lib/core/utils";

    let { class: className = "", solid = false }: { class?: string; solid?: boolean } = $props();
</script>

<span class={cn("inline-flex items-center", className)}>
    <img src="/favicon.png" alt="" draggable="false" class="mr-2 size-6" />
    <span class="wordmark text-[1.35rem] leading-none" class:solid>Deadlock<span class="plus">+</span></span>
</span>

<style>
    .wordmark {
        font-family: "Valve Pulp", var(--font-display);
        font-weight: 700;
        letter-spacing: 0.08em;
        background: linear-gradient(180deg, oklch(0.95 0.05 90), var(--brass) 60%, oklch(0.68 0.11 70));
        -webkit-background-clip: text;
        background-clip: text;
        color: transparent;
        filter: drop-shadow(0 0 10px color-mix(in oklch, var(--brass) 35%, transparent));
    }

    .wordmark.solid {
        background: none;
        color: var(--brass);
        -webkit-text-fill-color: var(--brass);
        filter: none;
    }

    .plus {
        background: none;
        color: var(--primary);
        -webkit-text-fill-color: var(--primary);
    }
</style>
