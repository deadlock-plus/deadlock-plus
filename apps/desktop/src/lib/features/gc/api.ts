import { command } from "$lib/core/tauri";

import type { GcStatus } from "$lib/generated/types/GcStatus";

export const gcStatus = () => command<GcStatus>("gc_status");
