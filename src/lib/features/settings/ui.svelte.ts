import type { CategoryId } from "./catalog";

class SettingsUi {
    open = $state(false);
    category = $state<CategoryId>("appearance");
    query = $state("");
    logsExpanded = $state(false);

    show(category?: CategoryId) {
        if (category) this.category = category;
        this.query = "";
        this.open = true;
    }

    close() {
        this.open = false;
    }
}

export const settingsUi = new SettingsUi();
