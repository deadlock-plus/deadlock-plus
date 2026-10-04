import { errorText } from "$lib/core/errors";
import { createPoller } from "$lib/core/poller";
import { frameCaptureStatus, startFrameCapture, stopFrameCapture, type CaptureStatus, type FrameStats } from "./api";

const POLL_MS = 500;

class FrameCaptureStore {
    status = $state<CaptureStatus | null>(null);
    result = $state<FrameStats | null>(null);
    error = $state<string | null>(null);
    active = $state(false);

    private poller = createPoller(() => this.poll(), { intervalMs: POLL_MS });

    async start() {
        if (this.active) return;
        this.error = null;
        this.result = null;
        try {
            await startFrameCapture();
            this.active = true;
            await this.poll();
            this.poller.start();
        } catch (e) {
            this.error = errorText(e);
        }
    }

    async stop() {
        if (!this.active) return;
        this.poller.stop();
        this.active = false;
        try {
            this.result = await stopFrameCapture();
        } catch (e) {
            this.error = errorText(e);
        }
        this.status = null;
    }

    private async poll() {
        try {
            this.status = await frameCaptureStatus();
        } catch (e) {
            this.error = errorText(e);
        }
    }
}

export const frameCapture = new FrameCaptureStore();
