export type CloseOutcome = "cancel" | "none";

/** Any close that did not follow a confirm click (Esc, overlay, navigation) counts as cancel. */
export function closeOutcome(nextOpen: boolean, confirmed: boolean): CloseOutcome {
    if (nextOpen || confirmed) return "none";
    return "cancel";
}
