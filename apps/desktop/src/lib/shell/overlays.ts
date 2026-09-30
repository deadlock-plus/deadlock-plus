export type CloseFn = () => void;

export function createOverlays() {
    const entries = new Set<{ close: CloseFn }>();

    return {
        register(close: CloseFn): () => void {
            const entry = { close };
            entries.add(entry);
            return () => void entries.delete(entry);
        },
        closeAll() {
            for (const entry of [...entries]) entry.close();
        },
        get size() {
            return entries.size;
        },
    };
}

export const overlays = createOverlays();

/** Wraps a callback so its first call is dropped. SvelteKit fires `afterNavigate` once on app start. */
export function skipFirst<A extends unknown[]>(fn: (...args: A) => void): (...args: A) => void {
    let first = true;
    return (...args) => {
        if (first) {
            first = false;
            return;
        }
        fn(...args);
    };
}
