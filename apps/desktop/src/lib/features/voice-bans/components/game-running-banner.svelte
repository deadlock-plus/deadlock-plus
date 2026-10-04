<script lang="ts">
    import { TriangleAlert } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
</script>

<div
    role="alert"
    class="flex items-center gap-4 rounded-lg border-2 border-destructive/60 bg-destructive/10 px-5 py-4 text-destructive"
>
    <TriangleAlert class="size-8 shrink-0" aria-hidden="true" />
    <div>
        <p class="font-heading text-lg font-semibold">{t("voice_bans.game_running.title")}</p>
        <p class="text-sm">
            {t("voice_bans.game_running.body")}
        </p>
    </div>
</div>
