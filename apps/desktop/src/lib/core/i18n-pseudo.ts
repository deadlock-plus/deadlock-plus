import type { Catalog } from "./i18n.svelte";

export const PSEUDO_LOCALE = "en-XA";

const ACCENTS: Record<string, string> = {
    a: "å",
    b: "ƀ",
    c: "ç",
    d: "ð",
    e: "é",
    f: "ƒ",
    g: "ĝ",
    h: "ĥ",
    i: "î",
    j: "ĵ",
    k: "ķ",
    l: "ļ",
    m: "ɱ",
    n: "ñ",
    o: "ö",
    p: "þ",
    q: "ǫ",
    r: "ŕ",
    s: "š",
    t: "ţ",
    u: "û",
    v: "ṽ",
    w: "ŵ",
    x: "ẋ",
    y: "ý",
    z: "ž",
    A: "Å",
    B: "Ɓ",
    C: "Ç",
    D: "Ð",
    E: "É",
    F: "Ƒ",
    G: "Ĝ",
    H: "Ĥ",
    I: "Î",
    J: "Ĵ",
    K: "Ķ",
    L: "Ļ",
    M: "Ṁ",
    N: "Ñ",
    O: "Ö",
    P: "Þ",
    Q: "Ǫ",
    R: "Ŕ",
    S: "Š",
    T: "Ţ",
    U: "Û",
    V: "Ṽ",
    W: "Ŵ",
    X: "Ẋ",
    Y: "Ý",
    Z: "Ž",
};

/** Matches `{name}` holes so they pass through untouched. */
const HOLE = /(\{\w+\})/;

/** Accents letters, pads by roughly 40% and wraps in brackets so clipped text is visible. */
export function pseudoText(text: string): string {
    const accented = text
        .split(HOLE)
        .map((part) => (HOLE.test(part) ? part : part.replace(/[A-Za-z]/g, (c) => ACCENTS[c] ?? c)))
        .join("");
    const padding = "~".repeat(Math.max(2, Math.ceil(text.replace(/\{\w+\}/g, "").length * 0.4)));
    return `[${accented} ${padding}]`;
}

export function pseudoLocalize(catalog: Catalog): Catalog {
    const out: Catalog = {};
    for (const [key, value] of Object.entries(catalog)) {
        out[key] = typeof value === "string" ? pseudoText(value) : pseudoLocalize(value);
    }
    return out;
}
