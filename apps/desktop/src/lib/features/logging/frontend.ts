import { debug, error, info, warn } from "$lib/core/log";
import { reportWebviewCrash } from "$lib/features/crash/api";
import { formatArgs, forwardConsole } from "./logs";

type ErrorLike = Pick<ErrorEvent, "message" | "filename" | "lineno" | "colno" | "error">;
type Target = Pick<EventTarget, "addEventListener" | "removeEventListener">;

const BENIGN = /^ResizeObserver loop/;

/**
 * Whether an uncaught error should leave a crash marker for the next launch. Only errors thrown as
 * real `Error` objects count, minus the browser's ResizeObserver notice. Unhandled rejections are
 * logged but never count: most are failed network or IPC calls that the app survives.
 */
export function isFatalError(e: Pick<ErrorEvent, "message" | "error">): boolean {
    return e.error != null && !BENIGN.test(e.message);
}

export function fatalErrorMessage(e: ErrorLike): string {
    const location = `${e.filename}:${e.lineno}:${e.colno}`;
    const stack = e.error instanceof Error && e.error.stack ? `\n${e.error.stack}` : "";
    return `Uncaught error: ${e.message} (${location})${stack}`;
}

/** Routes console output, uncaught errors and unhandled rejections into the app log file. */
export function installFrontendLogging(
    target: Target = window,
    report: (message: string) => Promise<unknown> = reportWebviewCrash,
): () => void {
    const restoreConsole = forwardConsole(console, { error, warn, info, debug });
    let reported = false;

    const onError = (e: ErrorEvent) => {
        error(`Uncaught error: ${e.message} (${e.filename}:${e.lineno}:${e.colno})`).catch(() => {});
        if (reported || !isFatalError(e)) return;
        reported = true;
        report(fatalErrorMessage(e)).catch(() => {});
    };
    const onRejection = (e: PromiseRejectionEvent) => {
        error(`Unhandled rejection: ${formatArgs([e.reason])}`).catch(() => {});
    };
    target.addEventListener("error", onError as EventListener);
    target.addEventListener("unhandledrejection", onRejection as EventListener);

    return () => {
        restoreConsole();
        target.removeEventListener("error", onError as EventListener);
        target.removeEventListener("unhandledrejection", onRejection as EventListener);
    };
}
