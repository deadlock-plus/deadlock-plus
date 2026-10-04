<script lang="ts">
    import { Search, X } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";

    type Props = {
        query: string;
        oninput: () => void;
        onclear: () => void;
    };

    let { query = $bindable(), oninput, onclear }: Props = $props();
    let input: HTMLInputElement | undefined = $state();
</script>

<header class="flex flex-col gap-4">
    <div>
        <h1 class="text-3xl">{t("alerts.title")}</h1>
        <p class="mt-1 text-base text-muted-foreground">{t("alerts.subtitle")}</p>
    </div>
    <div class="relative w-full">
        <Search
            class="pointer-events-none absolute left-4 top-1/2 size-5 -translate-y-1/2 text-muted-foreground"
            aria-hidden="true"
        />
        <input
            type="text"
            aria-label={t("alerts.search_aria")}
            bind:this={input}
            bind:value={query}
            {oninput}
            placeholder={t("alerts.search_placeholder")}
            class="h-14 w-full rounded-xl border border-border bg-card pl-12 pr-12 text-lg outline-none focus-visible:ring-2 focus-visible:ring-brass/50"
        />
        {#if query}
            <button
                type="button"
                aria-label={t("alerts.clear_search")}
                class="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                onclick={() => {
                    onclear();
                    input?.focus();
                }}
            >
                <X class="size-5" aria-hidden="true" />
            </button>
        {/if}
    </div>
</header>
