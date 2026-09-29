import { frameCaptureStatus, startFrameCapture, stopFrameCapture, type CaptureStatus, type FrameStats } from "./api";

const POLL_MS = 500;

class FrameCaptureStore {
    status = $state<CaptureStatus | null>(null);
    result = $state<FrameStats | null>(null);
    error = $state<string | null>(null);
    active = $state(false);

    private timer: ReturnType<typeof setInterval> | null = null;

    async start() {
        if (this.active) return;
        this.error = null;
        this.result = null;
        try {
            await startFrameCapture();
            this.active = true;
            await this.poll();
            this.timer = setInterval(() => void this.poll(), POLL_MS);
        } catch (e) {
            this.error = String(e);
        }
    }

    async stop() {
        if (!this.active) return;
        if (this.timer) clearInterval(this.timer);
        this.timer = null;
        this.active = false;
        try {
            this.result = await stopFrameCapture();
        } catch (e) {
            this.error = String(e);
        }
        this.status = null;
    }

    private async poll() {
        try {
            this.status = await frameCaptureStatus();
        } catch (e) {
            this.error = String(e);
        }
    }
}

export const frameCapture = new FrameCaptureStore();
