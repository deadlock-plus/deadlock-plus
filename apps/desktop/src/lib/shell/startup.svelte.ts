import { onMount } from "svelte";
import { afterNavigate } from "$app/navigation";
import { command, listen } from "$lib/core/tauri";
import { SERVICES, resolveReducedMotion, settings, updater } from "$lib/features/registry";
import { overlays, skipFirst } from "./overlays";

/** Call once from the root layout's script. Owns service start-up, document attributes and overlay lifecycle. */
export function useAppShell() {
    let osReducedMotion = $state(false);

    $effect(() => {
        document.documentElement.toggleAttribute("data-accessible-font", settings.accessibleFont);
    });

    $effect(() => {
        document.documentElement.dataset.theme = settings.theme;
    });

    $effect(() => {
        document.documentElement.toggleAttribute(
            "data-reduced-motion",
            resolveReducedMotion(settings.motion, osReducedMotion),
        );
    });

    $effect(() => {
        const available = updater.phase === "available" || updater.phase === "downloading";
        command("set_update_badge", { available }).catch(() => {});
    });

    // The first navigation is the app's own start-up; closing overlays then would close the ones it just opened.
    afterNavigate(skipFirst(() => overlays.closeAll()));

    onMount(() => {
        const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
        osReducedMotion = motionQuery.matches;
        const onMotionChange = (e: MediaQueryListEvent) => (osReducedMotion = e.matches);
        motionQuery.addEventListener("change", onMotionChange);

        const stops = SERVICES.map((start) => start());

        const unlistenHidden = listen("window-hidden", () => overlays.closeAll()).catch(() => () => {});

        // Two rAFs: the first fires before the browser has painted this frame, the second
        // guarantees one already happened. The window is built hidden so it's only ever revealed
        // with a real frame already rendered behind it, not a flash of empty/background-colored space.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                command("frontend_ready").catch((e) => console.error("frontend_ready failed:", e));
            });
        });

        return () => {
            motionQuery.removeEventListener("change", onMotionChange);
            for (const stop of stops) if (typeof stop === "function") stop();
            void unlistenHidden.then((fn) => fn());
        };
    });
}
