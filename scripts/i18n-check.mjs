import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PLURAL_SUFFIX = /_(zero|one|two|few|many|other)$/;
const KEY_CALL = /(?<![\w$])(tn?)\(\s*(["'])([\w.]+)\2/g;
const PLACEHOLDER = /\{(\w+)\}/g;
const DYNAMIC_PREFIXES = ["errors."];

export function flatten(node, prefix = "") {
    const out = {};
    for (const [name, value] of Object.entries(node)) {
        const key = prefix ? `${prefix}.${name}` : name;
        if (typeof value === "string") out[key] = value;
        else Object.assign(out, flatten(value, key));
    }
    return out;
}

export function extractKeys(source) {
    return [...source.matchAll(KEY_CALL)].map((m) => ({ key: m[3], plural: m[1] === "tn" }));
}

const placeholders = (text) => [...new Set([...text.matchAll(PLACEHOLDER)].map((m) => m[1]))].sort().join(",");

export function checkCatalogs(catalogs, used) {
    const errors = [];
    const warnings = [];
    const en = flatten(catalogs.en);

    for (const { file, key, plural } of used) {
        if (plural) {
            if (!(`${key}_other` in en)) errors.push(`${file} uses plural ${key} but en has no ${key}_other`);
        } else if (!(key in en)) {
            errors.push(`${file} uses missing key ${key}`);
        }
    }

    for (const [locale, catalog] of Object.entries(catalogs)) {
        if (locale === "en") continue;
        for (const [key, text] of Object.entries(flatten(catalog))) {
            const reference =
                en[key] ?? (PLURAL_SUFFIX.test(key) ? en[key.replace(PLURAL_SUFFIX, "_other")] : undefined);
            if (reference === undefined) errors.push(`${locale}: ${key} is not in en`);
            else if (placeholders(text) !== placeholders(reference))
                errors.push(`${locale}: ${key} placeholders differ from en`);
        }
    }

    const usedKeys = new Set(used.map((u) => u.key));
    for (const key of Object.keys(en)) {
        if (DYNAMIC_PREFIXES.some((p) => key.startsWith(p))) continue;
        if (!usedKeys.has(key) && !usedKeys.has(key.replace(PLURAL_SUFFIX, ""))) warnings.push(`unused en key ${key}`);
    }
    return { errors, warnings };
}

function walk(dir) {
    return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) return entry.name === "generated" ? [] : walk(full);
        return /\.(ts|svelte)$/.test(entry.name) && !/\.test\./.test(entry.name) ? [full] : [];
    });
}

function main() {
    const root = fileURLToPath(new URL("..", import.meta.url));
    const localesDir = path.join(root, "locales");
    const catalogs = Object.fromEntries(
        readdirSync(localesDir)
            .filter((f) => f.endsWith(".json"))
            .map((f) => [path.basename(f, ".json"), JSON.parse(readFileSync(path.join(localesDir, f), "utf8"))]),
    );
    const srcDir = path.join(root, "apps/desktop/src");
    const used = walk(srcDir).flatMap((full) =>
        extractKeys(readFileSync(full, "utf8")).map((k) => ({
            ...k,
            file: path.relative(root, full).replaceAll("\\", "/"),
        })),
    );
    const { errors, warnings } = checkCatalogs(catalogs, used);
    for (const line of warnings) console.warn(`warning: ${line}`);
    for (const line of errors) console.error(line);
    if (errors.length) process.exit(1);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
