<script lang="ts">
    import { toast } from "svelte-sonner";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { ExternalLink, Info, Scale } from "@lucide/svelte";
    import Badge from "$lib/components/ui/badge.svelte";
    import { APP_LICENSE, DISCLAIMER, LICENSES } from "$lib/features/settings/licenses";
    import DependencyLicenses from "$lib/features/settings/components/dependency-licenses.svelte";

    let { show }: { show: (id: string) => boolean } = $props();

    function open(url: string) {
        openUrl(url).catch((e) => toast.error(`Could not open the link: ${e}`));
    }
</script>

{#if show("licenses")}
    <div class="flex flex-col gap-4">
        <section class="rounded-lg border bg-card p-5">
            <div class="flex items-start gap-4">
                <div class="flex size-10 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
                    <Scale class="size-5" />
                </div>
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
                        <h2 class="font-heading text-base font-semibold tracking-wide">{APP_LICENSE.name}</h2>
                        <Badge variant="success">{APP_LICENSE.spdx}</Badge>
                    </div>
                    <p class="mt-2 text-sm text-muted-foreground">{APP_LICENSE.summary} {APP_LICENSE.warranty}</p>
                    <button
                        type="button"
                        class="mt-3 inline-flex items-center gap-1.5 text-sm text-primary hover:underline"
                        onclick={() => open(APP_LICENSE.url)}
                    >
                        Read the full licence
                        <ExternalLink class="size-3.5" />
                    </button>
                </div>
            </div>
        </section>

        <section class="flex items-start gap-3 rounded-lg border border-dashed px-4 py-3 text-sm text-muted-foreground">
            <Info class="mt-0.5 size-4 shrink-0" />
            <p>{DISCLAIMER}</p>
        </section>

        <section>
            <h3 class="mb-2 px-1 text-xs uppercase tracking-widest text-muted-foreground/70">Fonts and artwork</h3>
            <ul class="grid gap-3 sm:grid-cols-2">
                {#each LICENSES as license (license.name)}
                    <li class="flex flex-col rounded-lg border bg-card p-4">
                        <div class="flex items-start justify-between gap-3">
                            <div class="min-w-0">
                                <p class="text-sm font-semibold">{license.name}</p>
                                <p class="text-xs text-muted-foreground">{license.kind}</p>
                            </div>
                            <Badge variant={license.tone === "open" ? "success" : "warning"}>{license.license}</Badge>
                        </div>
                        <p class="mt-3 text-xs text-muted-foreground">{license.owner}</p>
                        <p class="mt-1 text-xs text-muted-foreground/80">{license.terms}</p>
                        {#if license.url}
                            <button
                                type="button"
                                class="mt-2 inline-flex w-fit items-center gap-1 text-xs text-primary hover:underline"
                                onclick={() => open(license.url!)}
                            >
                                Licence terms
                                <ExternalLink class="size-3" />
                            </button>
                        {/if}
                        <details class="mt-3 border-t border-border/60 pt-2">
                            <summary class="cursor-pointer text-xs text-muted-foreground hover:text-foreground"
                                >Copyright notice</summary
                            >
                            <p class="mt-2 text-xs text-muted-foreground/80 select-text">{license.notice}</p>
                        </details>
                    </li>
                {/each}
            </ul>
        </section>

        <DependencyLicenses />
    </div>
{/if}
