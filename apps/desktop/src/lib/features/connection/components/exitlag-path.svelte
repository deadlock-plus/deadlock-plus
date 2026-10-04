<script lang="ts">
    import Badge from "$lib/ui/badge.svelte";
    import { formatNumber, t } from "$lib/core/i18n.svelte";
    import Section from "$lib/ui/section.svelte";
    import type { EndpointInfo } from "../types";

    type Props = {
        endpoints: EndpointInfo[];
    };

    let { endpoints }: Props = $props();
</script>

<Section title={t("connection.path.title")} titleClass="mb-2">
    <div class="flex flex-col gap-1.5">
        {#each endpoints as e (e.ip + e.port)}
            <div class="flex items-center gap-3 text-xs">
                <Badge variant={e.isExit ? "success" : "secondary"}
                    >{e.isExit ? t("connection.path.exit") : t("connection.path.entry")}</Badge
                >
                <span class="font-mono">{e.ip}:{e.port}</span>
                <span class="text-muted-foreground tabular-nums"
                    >{t("connection.pkt_per_second", { value: formatNumber(Math.round(e.pps)) })}</span
                >
                <span class="ml-auto tabular-nums"
                    >{e.ping.avg == null
                        ? "—"
                        : t("connection.ms", { value: formatNumber(Math.round(e.ping.avg)) })}</span
                >
            </div>
        {/each}
    </div>
</Section>
