<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { ExternalLink, Newspaper } from "@lucide/svelte";

    import Badge from "$lib/components/ui/badge.svelte";
    import { alerts } from "$lib/features/alerts/alerts.svelte";
    import { formatPublished, kindTone, safeExternalUrl, type Alert } from "$lib/features/alerts/alerts";
    import { settings } from "$lib/features/settings/settings.svelte";

    let brokenImages = $state<Set<string>>(new Set());

    // Rows keep their unread marker while the page is open; they clear when you leave.
    onMount(() => {
        void alerts.fetchNow();
        return () => void alerts.markAllRead();
    });

    function open(alert: Alert) {
        const url = safeExternalUrl(alert.link);
        if (!url) return toast.error("This update has no valid link.");
        openUrl(url).catch((e) => toast.error(`Could not open the link: ${e}`));
    }
</script>

<div class="mx-auto flex min-h-full max-w-5xl flex-col gap-5 px-8 pb-12 pt-8">
    <header>
        <h1 class="text-3xl">Updates</h1>
        <p class="mt-1 text-base text-muted-foreground">Recent Deadlock patch notes and Steam announcements.</p>
    </header>

    {#if alerts.items.length === 0}
        <div class="flex flex-1 flex-col items-center justify-center gap-1 text-center text-base text-muted-foreground">
            <p>No updates loaded.</p>
            <p>
                Check your connection, or {settings.updateAlerts
                    ? "reopen this page to retry"
                    : "turn on alerts in Settings to be notified of new ones"}.
            </p>
        </div>
    {:else}
        <ul class="flex flex-col gap-4">
            {#each alerts.items as item (item.id)}
                {@const date = formatPublished(item.published)}
                {@const showImage = item.image !== null && !brokenImages.has(item.id)}
                <li>
                    <button
                        type="button"
                        class="group flex w-full flex-col gap-4 rounded-lg border border-border bg-card p-4 text-left transition-colors hover:border-brass/50 hover:bg-accent/40"
                        onclick={() => open(item)}
                    >
                        <div class="flex items-stretch gap-5">
                            <div
                                class="hidden aspect-video w-72 shrink-0 overflow-hidden rounded-md bg-background sm:block"
                            >
                                {#if showImage}
                                    <img
                                        src={item.image}
                                        alt=""
                                        loading="lazy"
                                        class="size-full object-contain"
                                        onerror={() => (brokenImages = new Set(brokenImages).add(item.id))}
                                    />
                                {:else}
                                    <div class="flex size-full items-center justify-center text-muted-foreground/40">
                                        <Newspaper class="size-10" aria-hidden="true" />
                                    </div>
                                {/if}
                            </div>

                            <div class="flex min-w-0 flex-1 flex-col justify-center gap-3">
                                <div class="flex flex-wrap items-center gap-2">
                                    <Badge
                                        variant={kindTone(item.kind) === "notable" ? "warning" : "secondary"}
                                        class="px-3 py-1 text-base"
                                    >
                                        {item.kind}
                                    </Badge>
                                    {#if date}<span class="text-base text-muted-foreground">{date}</span>{/if}
                                    {#if !item.read}
                                        <span class="ml-auto flex items-center gap-1.5 text-sm font-medium text-brass">
                                            <span class="size-2 rounded-full bg-brass" aria-hidden="true"></span>New
                                        </span>
                                    {/if}
                                </div>

                                <h2
                                    class="font-heading text-3xl font-semibold leading-tight tracking-wide text-foreground"
                                >
                                    {item.title}
                                </h2>

                                <span class="flex items-center gap-1.5 text-base font-medium text-brass">
                                    Read full notes <ExternalLink class="size-4" />
                                </span>
                            </div>
                        </div>

                        {#if item.summary}
                            <p class="line-clamp-6 whitespace-pre-line text-base leading-relaxed text-muted-foreground">
                                {item.summary}
                            </p>
                        {/if}
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>
