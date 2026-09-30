import { command } from "$lib/core/tauri";

export const changelog = () => command<string>("changelog");
