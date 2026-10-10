<script lang="ts">
    import { page } from "$app/state";
    import { ArrowLeft, LoaderCircle } from "@lucide/svelte";

    import { t } from "$lib/core/i18n.svelte";
    import Badge from "$lib/ui/badge.svelte";
    import EmptyState from "$lib/ui/empty-state.svelte";
    import Page from "$lib/ui/page.svelte";
    import DeepDive from "$lib/features/match-history/components/detail/deep-dive.svelte";
    import Scoreboard from "$lib/features/match-history/components/detail/scoreboard.svelte";
    import { headerSummary } from "$lib/features/match-history/components/detail/scoreboard";
    import Summary from "$lib/features/match-history/components/detail/summary.svelte";
    import { matchHistory } from "$lib/features/match-history/history.svelte";
    import { steamAccount } from "$lib/features/steam-account/account.svelte";

    const matchId = $derived(Number(page.params.id));
    const valid = $derived(Number.isSafeInteger(matchId) && matchId > 0);
    const ownAccountId = $derived(steamAccount.account?.steamId32 ?? null);
    const ownAccountIds = $derived(ownAccountId === null ? [] : [ownAccountId]);
    const detail = $derived(matchHistory.detailStatus === "ready" ? matchHistory.detail : null);

    $effect(() => {
        if (!valid) return;
        void matchHistory.openDetail(matchId);
        return () => matchHistory.closeDetail();
    });
</script>

<Page class="max-w-[1232px] px-4">
    <div>
        <a
            href="/match-history"
            class="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
        >
            <ArrowLeft class="size-4" aria-hidden="true" />
            {t("match_history.detail.back")}
        </a>
        <div class="mt-2 flex flex-wrap items-center gap-3">
            <h1 class="text-3xl">{t("match_history.detail.title", { id: valid ? matchId : "?" })}</h1>
            {#if detail?.source === "provisional"}
                <Badge variant="warning" title={t("match_history.list.provisional_hint")}>
                    {t("match_history.list.provisional")}
                </Badge>
            {/if}
        </div>
    </div>

    {#if !valid}
        <EmptyState size="base" spacing="xl" role="alert">{t("match_history.detail.invalid")}</EmptyState>
    {:else if matchHistory.detailStatus === "error"}
        <EmptyState size="base" spacing="xl" tone="destructive" role="alert">
            {t("match_history.detail.error", { error: matchHistory.detailError ?? "" })}
        </EmptyState>
    {:else if matchHistory.detailStatus === "unavailable"}
        <EmptyState size="base" spacing="xl" role="status">{t("match_history.detail.unavailable")}</EmptyState>
    {:else if detail}
        <Summary summary={headerSummary(detail, ownAccountId)} />
        <Scoreboard {detail} {ownAccountId} />
        <DeepDive {detail} {ownAccountIds} />
    {:else}
        <EmptyState size="base" spacing="xl" role="status" class="flex items-center justify-center gap-2">
            <LoaderCircle class="size-4 animate-spin" aria-hidden="true" />
            {t("match_history.detail.loading")}
        </EmptyState>
    {/if}
</Page>
