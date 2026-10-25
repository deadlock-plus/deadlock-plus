<script lang="ts">
    import type { Component } from "svelte";

    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import HistoryPage from "../+page.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";
    import { loadFrontPage, MATCH_ID, type FrontPage } from "$lib/proto/front-page/fixture";

    type VariantProps = {
        data: FrontPage;
        tossAt: number | null;
        still: boolean;
        onopen: () => void;
        onclose: () => void;
    };

    const modules = import.meta.glob<{ default: Component<VariantProps> }>("$lib/proto/front-page/variant-*.svelte");
    const variants = Object.keys(modules)
        .map((path) => {
            const match = /\/variant-([^/]+)\.svelte$/.exec(path);
            return match ? { id: `front-${match[1]}`, path } : null;
        })
        .filter((v) => v !== null)
        .sort((a, b) => a.id.localeCompare(b.id));

    const selected = $derived(page.url.searchParams.get("proto"));
    const current = $derived(variants.find((v) => v.id === selected));
    const loaded = $derived(current ? modules[current.path]() : null);
    const still = $derived(page.url.searchParams.get("still") === "1");
    const tossAt = $derived.by(() => {
        const raw = page.url.searchParams.get("t");
        return raw === null ? null : Number(raw);
    });

    const ownAccountId = $derived(steamAccount.account?.steamId32 ?? null);
    const data = $derived(loadFrontPage(ownAccountId));
</script>

<HistoryPage />

{#if current && loaded}
    {#await Promise.all([loaded, data]) then [module, front]}
        {@const Variant = module.default}
        <div data-proto={current.id}>
            <Variant
                data={front}
                {tossAt}
                {still}
                onopen={() => goto(`/match-history/${MATCH_ID}`)}
                onclose={() => goto("/match-history")}
            />
        </div>
    {:catch e}
        <p class="fixed bottom-4 left-4 z-[300] rounded bg-destructive p-2 text-destructive-foreground">{String(e)}</p>
    {/await}
{:else}
    <div class="fixed right-4 bottom-12 z-[300] space-y-1 rounded-md border bg-popover p-3 text-sm">
        <p class="font-semibold">Front page mockups</p>
        {#each variants as v (v.id)}
            <a class="block underline" href="?proto={v.id}">{v.id}</a>
        {/each}
    </div>
{/if}
