import type { Catalog } from "$lib/core/i18n.svelte";

export interface Language {
    code: string;
    /** The language written in itself, so it stays readable whatever the current locale is. */
    name: string;
    /** ISO 3166 country whose flag stands in for the language, or null for none. */
    flag: string | null;
}

export interface LanguageOption extends Language {
    available: boolean;
    completion: number;
}

export const LANGUAGES: readonly Language[] = [
    { code: "en", name: "English", flag: "GB" },
    { code: "af", name: "Afrikaans", flag: "ZA" },
    { code: "cs", name: "Čeština", flag: "CZ" },
    { code: "de", name: "Deutsch", flag: "DE" },
    { code: "es", name: "Español", flag: "ES" },
    { code: "fr", name: "Français", flag: "FR" },
    { code: "hu", name: "Magyar", flag: "HU" },
    { code: "id", name: "Bahasa Indonesia", flag: "ID" },
    { code: "it", name: "Italiano", flag: "IT" },
    { code: "ja", name: "日本語", flag: "JP" },
    { code: "ko", name: "한국어", flag: "KR" },
    { code: "pl", name: "Polski", flag: "PL" },
    { code: "pt", name: "Português", flag: "PT" },
    { code: "pt-BR", name: "Português (Brasil)", flag: "BR" },
    { code: "ru", name: "Русский", flag: "RU" },
    { code: "th", name: "ไทย", flag: "TH" },
    { code: "tr", name: "Türkçe", flag: "TR" },
    { code: "uk", name: "Українська", flag: "UA" },
    { code: "zh-CN", name: "简体中文", flag: "CN" },
    { code: "zh-TW", name: "繁體中文", flag: "TW" },
];

const PSEUDO: Language = { code: "en-XA", name: "Pseudo (dev)", flag: null };

const PLURAL_SUFFIX = /_(zero|one|two|few|many|other)$/;

function translatedGroups(node: Catalog, prefix = "", out = new Set<string>()): Set<string> {
    for (const [name, value] of Object.entries(node)) {
        const key = prefix ? `${prefix}.${name}` : name;
        if (typeof value === "string") {
            if (value.trim() !== "") out.add(key.replace(PLURAL_SUFFIX, ""));
        } else {
            translatedGroups(value, key, out);
        }
    }
    return out;
}

/** Whole percent of the reference's strings the catalog translates. Plural forms count as one string. */
export function completion(catalog: Catalog | undefined, reference: Catalog): number {
    const total = translatedGroups(reference);
    if (!catalog || total.size === 0) return 0;
    const done = translatedGroups(catalog);
    let count = 0;
    for (const key of total) if (done.has(key)) count++;
    return Math.floor((count / total.size) * 100);
}

export function languageOptions(
    supported: readonly string[],
    catalogs: Record<string, Catalog>,
    reference: Catalog,
): LanguageOption[] {
    const known = new Set(LANGUAGES.map((l) => l.code));
    const languages = [
        ...LANGUAGES,
        ...supported
            .filter((c) => !known.has(c))
            .map((c) => (c === PSEUDO.code ? PSEUDO : { code: c, name: c, flag: null })),
    ];
    return languages
        .map((language) => ({
            ...language,
            available: supported.includes(language.code),
            completion: language.code === "en" ? 100 : completion(catalogs[language.code], reference),
        }))
        .sort((a, b) => Number(b.available) - Number(a.available) || b.completion - a.completion);
}
