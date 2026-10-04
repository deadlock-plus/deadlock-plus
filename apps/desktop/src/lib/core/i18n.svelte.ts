import en from "../../../../../locales/en.json";
import { prefs } from "./prefs";

export type Catalog = { [key: string]: string | Catalog };
type Params = Record<string, string | number>;

export const SUPPORTED_LOCALES: readonly string[] = ["en"];
export const DEFAULT_LOCALE = "en";

/** Each language written in itself, so it stays recognisable whatever the current locale is. */
export const LOCALE_NAMES: Record<string, string> = { en: "English" };

const loaders: Record<string, () => Promise<{ default: Catalog }>> = {
    // Add `fr: () => import("../../../../../locales/fr.json")` style entries as locales ship.
};

export function lookup(catalog: Catalog, key: string): string | undefined {
    let node: string | Catalog | undefined = catalog;
    for (const part of key.split(".")) {
        if (typeof node !== "object") return undefined;
        node = node[part];
    }
    return typeof node === "string" ? node : undefined;
}

export function interpolate(template: string, params?: Params): string {
    return template.replace(/\{(\w+)\}/g, (hole, name) => (params && name in params ? String(params[name]) : hole));
}

export function resolveLocale(pref: string, system: string, supported: readonly string[]): string {
    const wanted = pref === "system" ? system : pref;
    if (supported.includes(wanted)) return wanted;
    const base = wanted.split("-")[0];
    return supported.includes(base) ? base : DEFAULT_LOCALE;
}

export function chainFor(locale: string, catalogs: Record<string, Catalog>): Catalog[] {
    const order = [locale, locale.split("-")[0], DEFAULT_LOCALE];
    const chain: Catalog[] = [];
    for (const code of order) {
        const catalog = catalogs[code];
        if (catalog && !chain.includes(catalog)) chain.push(catalog);
    }
    return chain;
}

class I18n {
    locale = $state(DEFAULT_LOCALE);
    catalogs = $state.raw<Record<string, Catalog>>({ en });

    reset(catalogs: Record<string, Catalog>, locale: string): void {
        this.catalogs = catalogs;
        this.locale = locale;
    }

    setLocale(locale: string): void {
        this.locale = locale;
        if (typeof document !== "undefined") document.documentElement.lang = locale;
    }

    async setLanguage(pref: string): Promise<string> {
        const system = typeof navigator === "undefined" ? DEFAULT_LOCALE : navigator.language;
        const locale = resolveLocale(pref, system, SUPPORTED_LOCALES);
        await this.load(locale);
        prefs.setString("language", pref);
        this.setLocale(locale);
        return locale;
    }

    private async load(locale: string): Promise<void> {
        const base = locale.split("-")[0];
        for (const code of new Set([locale, base])) {
            if (this.catalogs[code] || !loaders[code]) continue;
            const module = await loaders[code]();
            this.catalogs = { ...this.catalogs, [code]: module.default };
        }
    }

    translate(key: string, params?: Params): string {
        for (const catalog of chainFor(this.locale, this.catalogs)) {
            const template = lookup(catalog, key);
            if (template !== undefined) return interpolate(template, params);
        }
        if (import.meta.env?.DEV) console.warn(`[i18n] missing key: ${key}`);
        return key;
    }

    translatePlural(key: string, count: number, params?: Params): string {
        const form = new Intl.PluralRules(this.locale).select(count);
        const withCount = { ...params, count };
        for (const catalog of chainFor(this.locale, this.catalogs)) {
            const template = lookup(catalog, `${key}_${form}`) ?? lookup(catalog, `${key}_other`);
            if (template !== undefined) return interpolate(template, withCount);
        }
        if (import.meta.env?.DEV) console.warn(`[i18n] missing key: ${key}_${form}`);
        return key;
    }
}

export const i18n = new I18n();

export function t(key: string, params?: Params): string {
    return i18n.translate(key, params);
}

export function tn(key: string, count: number, params?: Params): string {
    return i18n.translatePlural(key, count, params);
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
    return new Intl.NumberFormat(i18n.locale, options).format(value);
}

export function formatDate(value: Date | number, options?: Intl.DateTimeFormatOptions): string {
    return new Intl.DateTimeFormat(i18n.locale, options).format(value);
}

export async function initLanguage(): Promise<string> {
    return i18n.setLanguage(prefs.getString("language", "system"));
}
