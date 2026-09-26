export interface ReportContext {
    route: string;
    at?: Date;
    status?: number;
}

export function errorSummary(error: unknown): string {
    if (typeof error === "string" && error) return error;
    if (error && typeof error === "object" && "message" in error) {
        const message = (error as { message: unknown }).message;
        if (typeof message === "string" && message) return message;
    }
    return "Unknown error";
}

export function errorReport(error: unknown, { route, at = new Date(), status }: ReportContext): string {
    const stack = error instanceof Error ? error.stack : undefined;
    const lines = [errorSummary(error), `Route: ${route}`];
    if (status !== undefined) lines.push(`Status: ${status}`);
    lines.push(`Time: ${at.toISOString()}`);
    if (stack) lines.push("", stack);
    return lines.join("\n");
}
