import { untrack } from "svelte";
import { overlays, type CloseFn } from "./overlays";

class OverlayHost {
    el = $state<HTMLElement | null>(null);
    count = $state(0);

    /** Marks one dialog as open. Returns the function that marks it closed again. */
    track(close: CloseFn): () => void {
        const forget = overlays.register(close);
        // Untracked: called from an effect that this counter also feeds through `inert`.
        untrack(() => this.count++);
        return () => {
            forget();
            untrack(() => this.count--);
        };
    }
}

export const overlayHost = new OverlayHost();
