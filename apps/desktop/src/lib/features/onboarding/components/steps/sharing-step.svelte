<script lang="ts">
    import Button from "$lib/ui/button.svelte";
    import { t } from "$lib/core/i18n.svelte";
    import type { IngestChoice } from "../../onboarding";

    let { choice, onAnswer }: { choice: IngestChoice | null; onAnswer: (choice: IngestChoice) => void } = $props();
</script>

<h1 class="font-heading text-2xl font-bold tracking-wide">{t("onboarding.sharing.title")}</h1>
<p class="text-muted-foreground">
    {t("onboarding.sharing.intro")}
</p>
<p class="text-sm text-muted-foreground">
    {t("onboarding.sharing.consent")}
</p>
<p class="text-sm text-muted-foreground">{t("onboarding.sharing.optional")}</p>
<div class="flex gap-2">
    <Button
        variant={choice === "share" ? "default" : "outline"}
        aria-pressed={choice === "share"}
        onclick={() => onAnswer("share")}
    >
        {t("onboarding.sharing.share")}
    </Button>
    <Button
        variant={choice === "decline" ? "default" : "outline"}
        aria-pressed={choice === "decline"}
        onclick={() => onAnswer("decline")}
    >
        {t("onboarding.sharing.decline")}
    </Button>
</div>
<p class="text-sm text-muted-foreground" role="status">
    {#if choice}
        {choice === "share" ? t("onboarding.sharing.saved_on") : t("onboarding.sharing.saved_off")}
    {:else}
        {t("onboarding.sharing.skipped")}
    {/if}
</p>
