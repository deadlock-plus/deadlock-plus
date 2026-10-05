import { render } from "./render.mjs";

const { DISCORD_UPDATES_WEBHOOK, RELEASE_TAG, RELEASE_URL, RELEASE_BODY } = process.env;

if (!DISCORD_UPDATES_WEBHOOK || !RELEASE_TAG || !RELEASE_URL) {
    console.error("DISCORD_UPDATES_WEBHOOK, RELEASE_TAG and RELEASE_URL are required");
    process.exit(1);
}

const target = new URL(DISCORD_UPDATES_WEBHOOK);
target.searchParams.set("with_components", "true");

const res = await fetch(target, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(render({ tag: RELEASE_TAG, url: RELEASE_URL, notes: RELEASE_BODY ?? "" })),
});

if (!res.ok) {
    console.error(`Discord rejected the message: ${res.status}`);
    process.exit(1);
}
console.log(`Announced ${RELEASE_TAG}`);
