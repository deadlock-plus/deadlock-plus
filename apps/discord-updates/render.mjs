const COMPONENTS_V2 = 1 << 15;
const ACCENT = 0x5865f2;
// Discord caps the text of one message at 4000 characters across all text displays.
const TEXT_BUDGET = 3600;
const MAX_LINE = 300;

function clip(text, max) {
    if (text.length <= max) return text;
    let cut = text.slice(0, max - 1);
    const last = cut.charCodeAt(cut.length - 1);
    if (last >= 0xd800 && last <= 0xdbff) cut = cut.slice(0, -1);
    return `${cut}…`;
}

/** Splits changelog notes into `###` sections of display lines. `####` groups become bold lines. */
export function sections(notes) {
    const out = [];
    let current;
    for (const raw of notes.split(/\r?\n/)) {
        const line = raw.trim();
        if (line === "") continue;
        if (line.startsWith("### ")) {
            current = { title: line.slice(4).trim(), lines: [] };
            out.push(current);
        } else if (current && line.startsWith("#### ")) {
            current.lines.push(`**${line.slice(5).trim()}**`);
        } else if (current) {
            current.lines.push(clip(line, MAX_LINE));
        }
    }
    return out;
}

function fit(list) {
    let left = TEXT_BUDGET;
    return list.map(({ title, lines }) => {
        const kept = [];
        let cut = 0;
        left -= title.length + 5;
        for (const line of lines) {
            if (left - line.length - 1 >= 60) {
                kept.push(line);
                left -= line.length + 1;
            } else {
                cut += 1;
            }
        }
        if (cut > 0) kept.push(`-# …and ${cut} more`);
        return { title, lines: kept };
    });
}

export function render({ tag, url, notes }) {
    const version = tag.replace(/^v/, "");
    const header = {
        type: 9,
        components: [{ type: 10, content: `## Deadlock+ ${version}\nA new version is out.` }],
        accessory: { type: 2, style: 5, label: "Release notes", url },
    };

    const blocks = fit(sections(notes)).map(({ title, lines }) => ({
        type: 10,
        content: [`### ${title}`, ...lines].join("\n"),
    }));

    const components = [header];
    if (blocks.length > 0) components.push({ type: 14, divider: true, spacing: 1 }, ...blocks);
    components.push(
        { type: 14, divider: true, spacing: 1 },
        {
            type: 10,
            content: "-# Deadlock+ checks for updates on its own. You can also download it from the release page.",
        },
    );

    return {
        username: "Deadlock+",
        flags: COMPONENTS_V2,
        allowed_mentions: { parse: [] },
        components: [{ type: 17, accent_color: ACCENT, components }],
    };
}
