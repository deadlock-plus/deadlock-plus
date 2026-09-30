import { command } from "$lib/core/tauri";

import type { LogEntry } from "./logs";

export const readLogs = () => command<LogEntry[]>("read_logs");
export const exportLogs = () => command<string>("export_logs");
export const openLogDir = () => command<void>("open_log_dir");
