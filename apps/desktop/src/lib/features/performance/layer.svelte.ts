import { errorText } from "$lib/core/errors";
import { frameLayerStatus, installFrameLayer, uninstallFrameLayer, type LayerStatus } from "./api";

class FrameLayerStore {
    status = $state<LayerStatus | null>(null);
    error = $state<string | null>(null);
    busy = $state(false);

    async load() {
        await this.run(frameLayerStatus);
    }

    async install() {
        await this.run(installFrameLayer);
    }

    async uninstall() {
        await this.run(uninstallFrameLayer);
    }

    private async run(action: () => Promise<LayerStatus>) {
        this.busy = true;
        this.error = null;
        try {
            this.status = await action();
        } catch (e) {
            this.error = errorText(e);
        } finally {
            this.busy = false;
        }
    }
}

export const frameLayer = new FrameLayerStore();
