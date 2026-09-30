<script lang="ts">
    import * as AlertDialog from "$lib/ui/alert-dialog";
    import { onboarding } from "$lib/features/onboarding/onboarding.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { whatsNew } from "../whats-new.svelte";
    import ReleaseNotes from "./release-notes.svelte";
</script>

<AlertDialog.Root
    open={whatsNew.open && !settings.ingestPromptPending && !onboarding.open}
    onOpenChange={(o) => !o && whatsNew.close()}
>
    <AlertDialog.Content class="max-w-lg">
        <div class="flex flex-col gap-3">
            <AlertDialog.Title>What's new</AlertDialog.Title>
            <div class="max-h-80 overflow-y-auto pr-1">
                <ReleaseNotes entries={whatsNew.entries} />
            </div>
        </div>
        <AlertDialog.Footer>
            <AlertDialog.Action onclick={() => whatsNew.close()}>Got it</AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
