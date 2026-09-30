<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import Button from "$lib/ui/button.svelte";
    import Card from "$lib/ui/card.svelte";
    import Page from "$lib/ui/page.svelte";
    import PageHeader from "$lib/ui/page-header.svelte";
    import ReleaseNotes from "$lib/features/updates/components/release-notes.svelte";
    import { isForcedExit, newestFirst } from "$lib/features/updates/whats-new";
    import { whatsNew } from "$lib/features/updates/whats-new.svelte";

    let sentinel = $state<HTMLElement | null>(null);
    let reachedEnd = $state(false);

    const forced = $derived(isForcedExit(page.url.searchParams));
    const releases = $derived(newestFirst(whatsNew.history));

    onMount(() => {
        if (!sentinel) return;
        const observer = new IntersectionObserver((hits) => {
            if (!hits.some((h) => h.isIntersecting)) return;
            reachedEnd = true;
            void whatsNew.markSeen();
            observer.disconnect();
        });
        observer.observe(sentinel);
        return () => observer.disconnect();
    });
</script>

<Page>
    <PageHeader title="What's new" subtitle="Release notes for Deadlock+, newest first." />

    {#if releases.length === 0}
        <Card as="section">
            <p class="text-sm text-muted-foreground">No release notes yet.</p>
        </Card>
    {:else}
        {#each releases as release (release.version)}
            <Card as="section" padding="lg">
                <ReleaseNotes entries={[release]} />
            </Card>
        {/each}
    {/if}

    <div bind:this={sentinel} class="flex min-h-9 justify-end pt-2">
        {#if forced && reachedEnd}
            <Button onclick={() => goto("/")}>Back to Home</Button>
        {/if}
    </div>
</Page>
