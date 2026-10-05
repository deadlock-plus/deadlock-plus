import assert from "node:assert/strict";
import { test } from "node:test";
import { render } from "./render.mjs";

const NOTES = `### Added

#### Discord activity

- Deadlock+ can show on your Discord profile.
- Turn it on in Settings > Discord.

#### Editor

- Change every line of your activity.

### Changed

- Community translations are updated.

### Fixed

- Text in Cyrillic uses a matching font.`;

const release = {
    tag: "v0.8.0",
    url: "https://github.com/deadlock-plus/deadlock-plus/releases/tag/v0.8.0",
    notes: NOTES,
};

function texts(message) {
    const out = [];
    const walk = (components) => {
        for (const c of components) {
            if (c.type === 10) out.push(c.content);
            if (c.components) walk(c.components);
        }
    };
    walk(message.components);
    return out;
}

function total(message) {
    return texts(message).reduce((n, t) => n + t.length, 0);
}

function count(components) {
    return components.reduce((n, c) => n + 1 + (c.components ? count(c.components) : 0) + (c.accessory ? 1 : 0), 0);
}

test("sends a Components V2 container that pings nobody", () => {
    const message = render(release);
    assert.equal(message.flags, 1 << 15);
    assert.deepEqual(message.allowed_mentions, { parse: [] });
    assert.equal(message.components.length, 1);
    assert.equal(message.components[0].type, 17);
});

test("header names the version without the tag prefix and links to the release", () => {
    const [container] = render(release).components;
    const header = container.components[0];
    assert.equal(header.type, 9);
    assert.match(header.components[0].content, /Deadlock\+ 0\.8\.0/);
    assert.equal(header.accessory.url, release.url);
});

test("each changelog section becomes one text block with its bullets", () => {
    const all = texts(render(release));
    const added = all.find((t) => t.startsWith("### Added"));
    assert.ok(added);
    assert.match(added, /\*\*Discord activity\*\*/);
    assert.match(added, /- Turn it on in Settings > Discord\./);
    assert.ok(all.some((t) => t.startsWith("### Changed") && t.includes("- Community translations")));
    assert.ok(all.some((t) => t.startsWith("### Fixed") && t.includes("Cyrillic")));
});

test("long notes stay under Discord's limits and say how many lines were cut", () => {
    const bullets = Array.from({ length: 200 }, (_, i) => `- Bullet number ${i} with some padding text to take room.`);
    const message = render({ ...release, notes: `### Added\n\n${bullets.join("\n")}` });
    assert.ok(total(message) <= 4000, `text was ${total(message)} characters`);
    assert.ok(count(message.components) <= 40);
    assert.match(texts(message).join("\n"), /…and \d+ more/);
});

test("one overlong bullet is clipped", () => {
    const message = render({ ...release, notes: `### Fixed\n\n- ${"x".repeat(2000)}` });
    assert.ok(total(message) < 1000);
});

test("empty notes still produce a header and the link", () => {
    const [container] = render({ ...release, notes: "" }).components;
    assert.equal(container.components[0].accessory.url, release.url);
});

test("a prerelease tag keeps its suffix", () => {
    const [container] = render({ ...release, tag: "v0.9.0-beta.1" }).components;
    assert.match(container.components[0].components[0].content, /0\.9\.0-beta\.1/);
});
