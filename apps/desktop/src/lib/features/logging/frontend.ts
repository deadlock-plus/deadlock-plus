import { debug, error, info, warn } from "$lib/core/log";
import { formatArgs, forwardConsole } from "./logs";

/** Routes console output, uncaught errors and unhandled rejections into the app log file. */
export function installFrontendLogging(): () => void {
    const restoreConsole = forwardConsole(console, { error, warn, info, debug });

    const onError = (e: ErrorEvent) => {
        error(`Uncaught error: ${e.message} (${e.filename}:${e.lineno}:${e.colno})`).catch(() => {});
    };
    const onRejection = (e: PromiseRejectionEvent) => {
        error(`Unhandled rejection: ${formatArgs([e.reason])}`).catch(() => {});
    };
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);

    return () => {
        restoreConsole();
        window.removeEventListener("error", onError);
        window.removeEventListener("unhandledrejection", onRejection);
    };
}
