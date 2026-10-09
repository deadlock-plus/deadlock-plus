import { command } from "$lib/core/tauri";

export const buildSupportReport = () => command<string>("build_support_report");
