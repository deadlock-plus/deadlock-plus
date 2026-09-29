<script lang="ts">
    import { Globe } from "@lucide/svelte";

    let { code }: { code: string | null } = $props();

    // Twemoji ships flags as SVG files named by their regional-indicator code points.
    // Windows has no flag emoji glyphs, so the images are the only way to render them.
    const files = import.meta.glob("/node_modules/@twemoji/svg/1f1*-1f1*.svg", {
        query: "?url",
        import: "default",
        eager: true,
    }) as Record<string, string>;

    function flagUrl(country: string): string | undefined {
        const points = [...country.toUpperCase()].map((c) => (0x1f1e6 + c.charCodeAt(0) - 65).toString(16));
        return files[`/node_modules/@twemoji/svg/${points.join("-")}.svg`];
    }

    const url = $derived(code ? flagUrl(code) : undefined);
</script>

{#if url}
    <img src={url} alt="" class="size-5 shrink-0" draggable="false" />
{:else}
    <Globe class="size-5 shrink-0 text-muted-foreground" />
{/if}
