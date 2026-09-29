import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const PACKAGE = "deadlock-plus";
const CARGO_VERSION = /^(version = ")[^"]*(")/m;

export function cargoVersion(text) {
    const pkg = text.split(/^\[(?!package\])/m)[0];
    return pkg.match(CARGO_VERSION)?.[0].split('"')[1];
}

export function withCargoVersion(text, version) {
    const end = text.search(/^\[(?!package\])/m);
    const head = end === -1 ? text : text.slice(0, end);
    return head.replace(CARGO_VERSION, `$1${version}$2`) + (end === -1 ? "" : text.slice(end));
}

const LOCK_ENTRY = new RegExp(`(name = "${PACKAGE}"\r?\nversion = ")([^"]*)(")`);

export function lockVersion(text) {
    return text.match(LOCK_ENTRY)?.[2];
}

export function withLockVersion(text, version) {
    return text.replace(LOCK_ENTRY, `$1${version}$3`);
}

function main() {
    const root = fileURLToPath(new URL("..", import.meta.url));
    const files = {
        cargo: `${root}apps/desktop/src-tauri/Cargo.toml`,
        lock: `${root}apps/desktop/src-tauri/Cargo.lock`,
    };
    const version = JSON.parse(readFileSync(`${root}apps/desktop/package.json`, "utf8")).version;
    const cargo = readFileSync(files.cargo, "utf8");
    const lock = readFileSync(files.lock, "utf8");
    const stale = [];
    if (cargoVersion(cargo) !== version) stale.push("apps/desktop/src-tauri/Cargo.toml");
    if (lockVersion(lock) !== version) stale.push("apps/desktop/src-tauri/Cargo.lock");

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
