import { platform } from "$lib/core/platform";

export function waitForPaint(): Promise<void> {
    // Hidden WebKitGTK views pause animation frames; WebView2 needs a painted frame before reveal.
    if (platform === "linux" && document.visibilityState === "hidden") return Promise.resolve();

    return new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
}
