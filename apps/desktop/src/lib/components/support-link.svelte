<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { t } from "$lib/core/i18n.svelte";
    import { openUrl } from "$lib/core/opener";
    import * as Tooltip from "$lib/ui/tooltip";
    import { SUPPORT_URL } from "$lib/features/support/support";

    let { collapsed }: { collapsed: boolean } = $props();

    const label = $derived(t("shell.sidebar.support"));

    const FIRST_PULSE_MS = 1500;
    const MIN_GAP_MS = 30_000;
    const MAX_GAP_MS = 45_000;

    let pulse = $state(0);

    onMount(() => {
        let timer: ReturnType<typeof setTimeout>;
        const schedule = (delay: number) => {
            timer = setTimeout(() => {
                if (document.visibilityState === "visible") pulse++;
                schedule(MIN_GAP_MS + Math.random() * (MAX_GAP_MS - MIN_GAP_MS));
            }, delay);
        };
        schedule(FIRST_PULSE_MS);
        return () => clearTimeout(timer);
    });

    function open() {
        openUrl(SUPPORT_URL).catch((e) => toast.error(t("common.open_link_failed", { error: String(e) })));
    }
</script>

<Tooltip.Provider>
    <Tooltip.Root delayDuration={100} disabled={!collapsed}>
        <Tooltip.Trigger>
            {#snippet child({ props })}
                <button
                    {...props}
                    type="button"
                    aria-label={label}
                    onclick={open}
                    class="flex h-10 w-full items-center gap-3 overflow-hidden rounded-md px-3.5 text-left font-heading text-sm font-semibold tracking-wide text-foreground transition-colors bg-kofi/10 hover:bg-kofi/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
                >
                    <span class="relative flex size-5 shrink-0 items-center justify-center text-kofi">
                        {#key pulse}
                            {#if pulse > 0}
                                <span class="pulse-ring absolute inset-0 rounded-full bg-kofi/40"></span>
                            {/if}
                        {/key}
                        <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" class="relative size-5">
                            <path
                                d="M11.351 2.715c-2.7 0-4.986.025-6.83.26C2.078 3.285 0 5.154 0 8.61c0 3.506.182 6.13 1.585 8.493 1.584 2.701 4.233 4.182 7.662 4.182h.83c4.209 0 6.494-2.234 7.637-4a9.5 9.5 0 0 0 1.091-2.338C21.792 14.688 24 12.22 24 9.208v-.415c0-3.247-2.13-5.507-5.792-5.87-1.558-.156-2.65-.208-6.857-.208m0 1.947c4.208 0 5.09.052 6.571.182 2.624.311 4.13 1.584 4.13 4v.39c0 2.156-1.792 3.844-3.87 3.844h-.935l-.156.649c-.208 1.013-.597 1.818-1.039 2.546-.909 1.428-2.545 3.064-5.922 3.064h-.805c-2.571 0-4.831-.883-6.078-3.195-1.09-2-1.298-4.155-1.298-7.506 0-2.181.857-3.402 3.012-3.714 1.533-.233 3.559-.26 6.39-.26m6.547 2.287c-.416 0-.65.234-.65.546v2.935c0 .311.234.545.65.545 1.324 0 2.051-.754 2.051-2s-.727-2.026-2.052-2.026m-10.39.182c-1.818 0-3.013 1.48-3.013 3.142 0 1.533.858 2.857 1.949 3.897.727.701 1.87 1.429 2.649 1.896a1.47 1.47 0 0 0 1.507 0c.78-.467 1.922-1.195 2.623-1.896 1.117-1.039 1.974-2.364 1.974-3.897 0-1.662-1.247-3.142-3.039-3.142-1.065 0-1.792.545-2.338 1.298-.493-.753-1.246-1.298-2.312-1.298"
                            />
                        </svg>
                    </span>
                    {#if !collapsed}
                        <span class="truncate whitespace-nowrap">{label}</span>
                    {/if}
                </button>
            {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content side="right">{label}</Tooltip.Content>
    </Tooltip.Root>
</Tooltip.Provider>

<style>
    .pulse-ring {
        opacity: 0;
        animation: support-pulse 1.6s ease-out 3;
    }

    @keyframes support-pulse {
        0% {
            transform: scale(0.8);
            opacity: 0.8;
        }
        100% {
            transform: scale(2);
            opacity: 0;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .pulse-ring {
            animation: none;
        }
    }
</style>
