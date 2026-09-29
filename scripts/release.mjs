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

/** GitHub rewrites some characters in asset names (the "+" in the product name), so the URL must not depend on them. */
export function assetName(version) {
    return `Deadlock-Plus_${version}_x64-setup.exe`;
}

export function updaterManifest({ version, notes, pubDate, signature, repo }) {
    return {
        version,
        notes,
        pub_date: pubDate,
        platforms: {
            "windows-x86_64": {
                signature,
                url: `https://github.com/${repo}/releases/download/v${version}/${assetName(version)}`,
            },
        },
    };
}

function prepare(repo) {
    const root = fileURLToPath(new URL("..", import.meta.url));
    const version = JSON.parse(readFileSync(`${root}apps/desktop/package.json`, "utf8")).version;
    const notes = changelogNotes(readFileSync(`${root}CHANGELOG.md`, "utf8"), version);
    if (!notes) throw new Error(`CHANGELOG.md has no entry for ${version}`);

    const bundle = `${root}apps/desktop/src-tauri/target/release/bundle/nsis`;
    const installer = readdirSync(bundle).find((f) => f.endsWith(`_${version}_x64-setup.exe`));
    if (!installer || !existsSync(`${bundle}/${installer}.sig`)) {
        throw new Error(`No signed installer for ${version} in ${bundle}`);
    }

    const out = `${root}release-out`;
    mkdirSync(out, { recursive: true });
    copyFileSync(`${bundle}/${installer}`, `${out}/${assetName(version)}`);
    const signature = readFileSync(`${bundle}/${installer}.sig`, "utf8").trim();
    const manifest = updaterManifest({ version, notes, pubDate: new Date().toISOString(), signature, repo });
    writeFileSync(`${out}/latest.json`, JSON.stringify(manifest, null, 2));
    writeFileSync(`${out}/notes.md`, notes);
    console.log(`Prepared ${out} for v${version}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
    const [command, repo] = process.argv.slice(2);
    if (command !== "prepare" || !repo) {
        console.error("Usage: node scripts/release.mjs prepare <owner/repo>");
        process.exit(1);
    }
    try {
        prepare(repo);
    } catch (e) {
        console.error(e.message);
        process.exit(1);
    }
}
