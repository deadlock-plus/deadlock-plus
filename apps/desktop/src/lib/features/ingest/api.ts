import { command } from "$lib/core/tauri";

import type { IngestStatus } from "$lib/generated/types/IngestStatus";

export const ingestStatus = () => command<IngestStatus>("ingest_status");
