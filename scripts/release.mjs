import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

/** Body of one `## [x.y.z]` entry, or null when the entry is missing or empty. */
export function changelogNotes(markdown, version) {
    const lines = markdown.split(/\r?\n/);
    const heading = new RegExp(`^##\\s+\\[?v?${version.replace(/[.+*?^${}()|[\]\\]/g, "\\$&")}\\]?(\\s|$)`);
    const start = lines.findIndex((l) => heading.test(l));
    if (start === -1) return null;
    let end = lines.findIndex((l, i) => i > start && /^##\s/.test(l));
    if (end === -1) end = lines.length;
    return (
        lines
            .slice(start + 1, end)
            .join("\n")
            .trim() || null
    );
}

const PLATFORMS = {
    "windows-x86_64": { bundleDir: "nsis", suffix: "_x64-setup.exe" },
    "linux-x86_64-appimage": { bundleDir: "appimage", suffix: "_amd64.AppImage" },
    "linux-x86_64-deb": { bundleDir: "deb", suffix: "_amd64.deb" },
};

const GROUPS = {
    windows: ["windows-x86_64"],
    linux: ["linux-x86_64-appimage", "linux-x86_64-deb"],
};

/** GitHub rewrites some characters in asset names (the "+" in the product name), so the URL must not depend on them. */
export function assetName(version, platform) {
    const entry = PLATFORMS[platform];
    if (!entry) throw new Error(`Unknown platform ${platform}`);
    return `Deadlock-Plus_${version}${entry.suffix}`;
}

export function updaterManifest({ version, notes, pubDate, signatures, repo }) {
    const platforms = {};
    for (const platform of Object.keys(PLATFORMS)) {
        const signature = signatures[platform];
        if (!signature) throw new Error(`No signature for ${platform}`);
        platforms[platform] = {
            signature,
            url: `https://github.com/${repo}/releases/download/v${version}/${assetName(version, platform)}`,
        };
    }
    return { version, notes, pub_date: pubDate, platforms };
}

function rootAndVersion() {
    const root = fileURLToPath(new URL("..", import.meta.url));
    const version = JSON.parse(readFileSync(`${root}apps/desktop/package.json`, "utf8")).version;
    return { root, version };
}

/** Renames one platform group's bundles to their release asset names and keeps each updater signature beside them. */
function prepare(group) {
    const platforms = GROUPS[group];
    if (!platforms) throw new Error(`Unknown group ${group}`);
    const { root, version } = rootAndVersion();
    const out = `${root}release-${group}`;
    mkdirSync(out, { recursive: true });
    for (const platform of platforms) {
        const dir = `${root}target/release/bundle/${PLATFORMS[platform].bundleDir}`;
        const bundle = existsSync(dir)
            ? readdirSync(dir).find((f) => f.endsWith(`_${version}${PLATFORMS[platform].suffix}`))
            : undefined;
        if (!bundle || !existsSync(`${dir}/${bundle}.sig`)) {
            throw new Error(`No signed ${platform} bundle for ${version} in ${dir}`);
        }
        copyFileSync(`${dir}/${bundle}`, `${out}/${assetName(version, platform)}`);
        copyFileSync(`${dir}/${bundle}.sig`, `${out}/${platform}.sig`);
    }
    console.log(`Prepared ${out} for v${version}`);
}

/** Merges the per-group outputs in the `parts` directory into one release-out with the updater feed. */
function manifest(repo, parts) {
    const { root, version } = rootAndVersion();
    const notes = changelogNotes(readFileSync(`${root}CHANGELOG.md`, "utf8"), version);
    if (!notes) throw new Error(`CHANGELOG.md has no entry for ${version}`);

    const out = `${root}release-out`;
    mkdirSync(out, { recursive: true });
    const signatures = {};
    for (const part of readdirSync(parts)) {
        for (const file of readdirSync(`${parts}/${part}`)) {
            if (file.endsWith(".sig")) {
                signatures[file.slice(0, -4)] = readFileSync(`${parts}/${part}/${file}`, "utf8").trim();
            } else {
                copyFileSync(`${parts}/${part}/${file}`, `${out}/${file}`);
            }
        }
    }
    const feed = updaterManifest({ version, notes, pubDate: new Date().toISOString(), signatures, repo });
    writeFileSync(`${out}/latest.json`, JSON.stringify(feed, null, 2));
    writeFileSync(`${out}/notes.md`, notes);
    console.log(`Prepared ${out} for v${version}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
    const [command, a, b] = process.argv.slice(2);
    try {
        if (command === "prepare" && a) prepare(a);
        else if (command === "manifest" && a && b) manifest(a, b);
        else {
            console.error("Usage: node scripts/release.mjs prepare <windows|linux>");
            console.error("       node scripts/release.mjs manifest <owner/repo> <parts-dir>");
            process.exit(1);
        }
    } catch (e) {
        console.error(e.message);
        process.exit(1);
    }
}
