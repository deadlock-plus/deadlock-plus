import en from "../../../../../locales/en.json";

type Catalog = { errors: Record<string, Record<string, string>> };

const GENERIC = ["common", "internal"] as const;

function lookup(catalog: Catalog, code: string): string | undefined {
    const [feature, name] = code.split(".");
    return catalog.errors[feature]?.[name];
}

function isAppError(value: unknown): value is { code: string; params?: Record<string, string> } {
    return typeof value === "object" && value !== null && typeof (value as { code?: unknown }).code === "string";
}

export function errorText(error: unknown, catalog: Catalog = en): string {
    if (typeof error === "string") return error;
    if (error instanceof Error) return error.message;
    if (isAppError(error)) {
        const template = lookup(catalog, error.code);
        if (template) return template.replace(/\{(\w+)\}/g, (hole, name) => error.params?.[name] ?? hole);
    }
    return catalog.errors[GENERIC[0]][GENERIC[1]];
}
