import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const CARGO_VERSION = /^(version = ")[^"]*(")/m;
const SECTION = /^\[workspace\.package\]\r?\n/m;
const SECTION_END = /^\[/m;

function packageSection(text) {
    const header = text.match(SECTION);
    if (!header) return undefined;
    const bodyStart = header.index + header[0].length;
    const rest = text.slice(bodyStart).search(SECTION_END);
    return { bodyStart, bodyEnd: rest === -1 ? text.length : bodyStart + rest };
}

export function cargoVersion(text) {
    const range = packageSection(text);
    if (!range) return undefined;
    return text.slice(range.bodyStart, range.bodyEnd).match(CARGO_VERSION)?.[0].split('"')[1];
}

export function withCargoVersion(text, version) {
    const range = packageSection(text);
    if (!range) return text;
    const body = text.slice(range.bodyStart, range.bodyEnd).replace(CARGO_VERSION, `$1${version}$2`);
    return text.slice(0, range.bodyStart) + body + text.slice(range.bodyEnd);
}

const LOCK_ENTRY = /(name = "(?:deadlock-plus|dp-[a-z0-9-]+)"\r?\nversion = ")([^"]*)(")/g;

export function lockVersions(text) {
    return [...text.matchAll(LOCK_ENTRY)].map((m) => m[2]);
}

export function lockVersion(text) {
    return lockVersions(text)[0];
}

export function withLockVersion(text, version) {
    return text.replace(LOCK_ENTRY, `$1${version}$3`);
}

function main() {
    const root = fileURLToPath(new URL("..", import.meta.url));
    const files = {
        cargo: `${root}Cargo.toml`,
        lock: `${root}Cargo.lock`,
    };
    const version = JSON.parse(readFileSync(`${root}apps/desktop/package.json`, "utf8")).version;
    const cargo = readFileSync(files.cargo, "utf8");
    const lock = readFileSync(files.lock, "utf8");
    const stale = [];
    if (cargoVersion(cargo) !== version) stale.push("Cargo.toml");
    if (!lockVersions(lock).every((v) => v === version)) stale.push("Cargo.lock");

    if (process.argv.includes("--check")) {
        if (stale.length) {
            console.error(`Version ${version} (package.json) is not in: ${stale.join(", ")}. Run pnpm version:sync.`);
            process.exit(1);
        }
        return;
    }

    writeFileSync(files.cargo, withCargoVersion(cargo, version));
    writeFileSync(files.lock, withLockVersion(lock, version));
    console.log(`Synced version ${version}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
