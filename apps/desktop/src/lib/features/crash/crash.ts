import type { CrashKind } from "$lib/generated/types/CrashKind";
import type { CrashReport } from "$lib/generated/types/CrashReport";

export type { CrashKind, CrashReport };

export {
    dismissCrash,
    openCrashIssue,
    pendingCrash,
    reportWebviewCrash,
    revealCrashBundle,
    writeCrashBundle,
} from "./api";
