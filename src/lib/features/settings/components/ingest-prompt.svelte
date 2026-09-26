<script lang="ts">
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { onboarding } from "$lib/features/onboarding/onboarding.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
</script>

<AlertDialog.Root open={settings.ingestPromptPending && !onboarding.open}>
    <AlertDialog.Content class="max-w-md">
        <div class="flex flex-col gap-3">
            <AlertDialog.Title>Help the Deadlock community?</AlertDialog.Title>
            <AlertDialog.Description>
                The Deadlock API is a free, community-run database behind many stat and replay sites. Sharing your
                matches helps keep it complete for every player.
            </AlertDialog.Description>
            <p class="text-sm text-muted-foreground">
                If you agree, Deadlock+ sends only the match IDs and replay keys Steam already stores on your PC, plus
                your Steam account ID. Nothing else leaves your computer.
            </p>
            <p class="text-sm text-muted-foreground">Optional. You can change this any time in Settings.</p>
        </div>
        <AlertDialog.Footer>
            <AlertDialog.Cancel onclick={() => settings.setMatchIngest(false)}>No thanks</AlertDialog.Cancel>
            <AlertDialog.Action onclick={() => settings.setMatchIngest(true)}>Share my matches</AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
