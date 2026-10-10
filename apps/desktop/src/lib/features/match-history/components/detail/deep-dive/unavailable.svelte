<script lang="ts">
    import { CircleSlash } from "@lucide/svelte";
    import { t } from "$lib/core/i18n.svelte";
</script>

<p class="flex items-center gap-2 text-sm text-muted-foreground">
    <CircleSlash size={16} class="shrink-0" aria-hidden="true" />
    {t("match_history.deep_dive.not_available")}
</p>
