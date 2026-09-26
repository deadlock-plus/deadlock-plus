export interface IngestConsent {
    enabled: boolean;
    needsPrompt: boolean;
}

// Uploading needs an explicit answer: an unanswered prompt keeps the watcher off.
export function resolveIngestConsent(stored: boolean | null | undefined): IngestConsent {
    if (stored === true) return { enabled: true, needsPrompt: false };
    if (stored === false) return { enabled: false, needsPrompt: false };
    return { enabled: false, needsPrompt: true };
}
