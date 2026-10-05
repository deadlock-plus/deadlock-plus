export interface TelemetryChoice {
    enabled: boolean;
    noticeShown: boolean;
}

export function resolveTelemetry(
    storedEnabled: boolean | null | undefined,
    storedNoticeShown: boolean | null | undefined,
): TelemetryChoice {
    return { enabled: storedEnabled !== false, noticeShown: storedNoticeShown === true };
}

/** Sending is opt-out, but only once the user has been told about it. */
export function shouldSend(choice: TelemetryChoice): boolean {
    return choice.enabled && choice.noticeShown;
}

/** New users see the notice in the onboarding step; everyone else gets a one-time toast. */
export function needsLaunchNotice(choice: TelemetryChoice, onboardingNeeded: boolean): boolean {
    return !choice.noticeShown && !onboardingNeeded;
}
