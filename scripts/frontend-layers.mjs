import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// [file relative to src/, rule]. Entries may only be removed; a fixed file must be dropped from here.
export const LAYER_ALLOWLIST = [
    ["lib/shell/sidebar.svelte", "shell-registry"],
    ["lib/shell/statusbar.svelte", "shell-registry"],
    ["lib/shell/titlebar.svelte", "shell-registry"],
];

const TEST_FILE = /\.(test|spec)\.[jt]s$/;
const TAURI_ALLOWED = /^lib\/core\/|^lib\/features\/[^/]+\/api\.ts$/;
const IMPORT_PATTERNS = [
    /\b(?:import|export)\b[^'"`;()]*?\bfrom\s*["']([^"']+)["']/g,
    /\bimport\s*["']([^"']+)["']/g,
    /\bimport\s*\(\s*["']([^"']+)["']\s*\)/g,
];

export function extractImports(source) {
    const found = [];
    for (const pattern of IMPORT_PATTERNS) {
        for (const match of source.matchAll(pattern)) found.push([match.index, match[1]]);
    }
    return found.sort((a, b) => a[0] - b[0]).map(([, specifier]) => specifier);
}

function resolveSpecifier(file, specifier) {
    if (specifier.startsWith("$lib/")) return `lib/${specifier.slice(5)}`;
    if (specifier.startsWith(".")) return path.posix.join(path.posix.dirname(file), specifier);
    return specifier;
}

function rulesBroken(file, specifier) {
    const target = resolveSpecifier(file, specifier);
    const broken = [];
    if (specifier.startsWith("@tauri-apps/") && !TEST_FILE.test(file) && !TAURI_ALLOWED.test(file)) {
        broken.push(["tauri", "only lib/core and lib/features/*/api.ts may use @tauri-apps"]);
    }
    const intoFeatures = target.startsWith("lib/features/") || target === "lib/features";
    if (intoFeatures && (file.startsWith("lib/core/") || file.startsWith("lib/ui/"))) {
        broken.push(["no-features", "lib/core and lib/ui must not import lib/features"]);
    }
    if (intoFeatures && file.startsWith("lib/shell/") && target !== "lib/features/registry") {
        broken.push(["shell-registry", "lib/shell may only reach features through lib/features/registry"]);
    }
    return broken;
}

export function checkLayers(files, allowlist = LAYER_ALLOWLIST) {
    const violations = [];
    const allowed = new Set(allowlist.map(([file, rule]) => `${file}\0${rule}`));
    const seen = new Set();

    for (const { file, imports } of files) {
        for (const specifier of imports) {
            for (const [rule, why] of rulesBroken(file, specifier)) {
                const key = `${file}\0${rule}`;
                if (allowed.has(key)) {
                    seen.add(key);
                    continue;
                }
                violations.push(`${file} imports ${specifier}: ${why}`);
            }
        }
    }

    for (const [file, rule] of allowlist) {
        if (!seen.has(`${file}\0${rule}`)) {
            violations.push(`stale allow-list entry ${file} (${rule}): no such violation`);
        }
    }

    return violations;
}

function* walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) yield* walk(full);
        else if (/\.(ts|svelte)$/.test(entry.name)) yield full;
    }
}

export function scan(srcDir) {
    return [...walk(srcDir)]
        .map((full) => path.relative(srcDir, full).replaceAll("\\", "/"))
        .filter((file) => !file.startsWith("lib/generated/"))
        .map((file) => ({ file, imports: extractImports(readFileSync(path.join(srcDir, file), "utf8")) }));
}

function main() {
    const srcDir = fileURLToPath(new URL("../apps/desktop/src", import.meta.url));
    const violations = checkLayers(scan(srcDir));
    if (violations.length) {
        for (const line of violations) console.error(line);
        process.exit(1);
    }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
