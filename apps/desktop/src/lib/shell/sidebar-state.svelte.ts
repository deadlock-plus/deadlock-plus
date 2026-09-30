import { prefs } from "$lib/core/prefs";

class SidebarState {
    collapsed = $state(prefs.getBool("sidebarCollapsed", false));

    toggle() {
        this.collapsed = !this.collapsed;
        prefs.setBool("sidebarCollapsed", this.collapsed);
    }
}

export const sidebarState = new SidebarState();
