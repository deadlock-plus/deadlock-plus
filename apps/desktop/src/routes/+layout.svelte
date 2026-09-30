<script lang="ts">
    import "../app.css";
    import { Toaster } from "svelte-sonner";
    import Titlebar from "$lib/shell/titlebar.svelte";
    import { useAppShell } from "$lib/shell/startup.svelte";
    import IngestPrompt from "$lib/features/settings/components/ingest-prompt.svelte";
    import OnboardingDialog from "$lib/features/onboarding/components/onboarding-dialog.svelte";
    import WhatsNewDialog from "$lib/features/updates/components/whats-new-dialog.svelte";
    import { settings } from "$lib/features/settings/settings.svelte";
    import { isLightTheme } from "$lib/features/settings/themes";

    let { children } = $props();

    useAppShell();
</script>

<div class="flex h-screen w-screen flex-col overflow-hidden bg-chrome text-foreground">
    <Titlebar />
    {@render children?.()}
</div>

<OnboardingDialog />
<IngestPrompt />
<WhatsNewDialog />

<Toaster richColors position="bottom-right" theme={isLightTheme(settings.theme) ? "light" : "dark"} />
