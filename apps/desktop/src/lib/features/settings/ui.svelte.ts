class SettingsUi {
    query = $state("");
    logsExpanded = $state(false);
    restoreLogsToggleFocus = false;
    /** The page the user came from, so "Back" leaves settings instead of stepping through categories. */
    returnTo: string | null = null;

    toggleLogsExpanded() {
        this.restoreLogsToggleFocus = true;
        this.logsExpanded = !this.logsExpanded;
    }
}

export const settingsUi = new SettingsUi();
