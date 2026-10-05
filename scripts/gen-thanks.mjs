// Refreshes the contributors and translators in apps/desktop/src/lib/features/thanks/thanks.json.
// Donators are never touched: they are opt-in and edited by hand.
// Env: GITHUB_TOKEN (optional, raises the rate limit), GITHUB_REPOSITORY (default below),
//      CROWDIN_PERSONAL_TOKEN + CROWDIN_PROJECT_ID (translators are kept as they are when unset).
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const DEFAULT_REPO = "deadlock-plus/deadlock-plus";
const EXCLUDE_LOGINS = ["crowdin-bot"];

// Crowdin language ids to the codes the app loads (see `languages_mapping` in crowdin.yml).
const LANGUAGE_MAP = {
    af: "af",
    cs: "cs",
    de: "de",
    "es-ES": "es",
    fr: "fr",
    hu: "hu",
    id: "id",
    it: "it",
    ja: "ja",
    ko: "ko",
    pl: "pl",
    "pt-PT": "pt",
    "pt-BR": "pt-BR",
    ru: "ru",
    th: "th",
    tr: "tr",
    uk: "uk",
    "zh-CN": "zh-CN",
    "zh-TW": "zh-TW",
};

export function appLanguage(crowdinId) {
    return LANGUAGE_MAP[crowdinId] ?? null;
}

export function contributorsFrom(rows, exclude = EXCLUDE_LOGINS) {
    const skip = new Set(exclude.map((l) => l.toLowerCase()));
    return rows
        .filter((r) => r.type === "User" && r.contributions > 0 && !skip.has(r.login.toLowerCase()))
        .sort((a, b) => b.contributions - a.contributions)
        .map((r) => ({ name: r.login, url: r.html_url }));
}

/** `rows` are the entries of a Crowdin "top members" report. Only the public username is used, never the full name. */
export function translatorsFrom(rows) {
    return rows
        .filter((r) => (r.translated ?? 0) + (r.approved ?? 0) > 0)
        .sort((a, b) => b.translated + b.approved - (a.translated + a.approved))
        .flatMap((r) => {
            const languages = [...new Set((r.languages ?? []).map((l) => appLanguage(l.id)).filter(Boolean))];
            if (languages.length === 0) return [];
            const name = r.user.username;
            return [{ name, url: `https://crowdin.com/profile/${encodeURIComponent(name)}`, languages }];
        });
}

/** A `null` list means "could not be fetched": the current one stays. */
export function mergeThanks(current, { contributors, translators }) {
    return {
        contributors: contributors ?? current.contributors,
        translators: translators ?? current.translators,
        donators: current.donators,
    };
}

async function json(url, init) {
    const res = await fetch(url, init);
    if (!res.ok) throw new Error(`${init?.method ?? "GET"} ${url}: ${res.status} ${await res.text()}`);
    return res.json();
}

async function fetchContributors(repo, token) {
    const headers = { Accept: "application/vnd.github+json", ...(token ? { Authorization: `Bearer ${token}` } : {}) };
    const rows = [];
    for (let page = 1; ; page++) {
        const batch = await json(`https://api.github.com/repos/${repo}/contributors?per_page=100&page=${page}`, {
            headers,
        });
        rows.push(...batch);
        if (batch.length < 100) return contributorsFrom(rows);
    }
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function fetchTranslators(projectId, token) {
    const api = `https://api.crowdin.com/api/v2/projects/${projectId}`;
    const headers = { Authorization: `Bearer ${token}`, "Content-Type": "application/json" };
    const created = await json(`${api}/reports`, {
        method: "POST",
        headers,
        body: JSON.stringify({
            name: "top-members",
            schema: {
                unit: "words",
                languageId: "all",
                format: "json",
                dateFrom: "2020-01-01T00:00:00+00:00",
                dateTo: new Date().toISOString(),
            },
        }),
    });
    const id = created.data.identifier;
    for (let attempt = 0; attempt < 30; attempt++) {
        const status = await json(`${api}/reports/${id}`, { headers });
        if (status.data.status === "finished") {
            const link = await json(`${api}/reports/${id}/download`, { headers });
            const report = await json(link.data.url);
            return translatorsFrom(report.data ?? []);
        }
        if (status.data.status === "failed") throw new Error("Crowdin report failed");
        await sleep(2000);
    }
    throw new Error("Crowdin report timed out");
}

async function attempt(label, fn) {
    try {
        return await fn();
    } catch (e) {
        console.warn(`${label}: ${e.message}; keeping the current list.`);
        return null;
    }
}

async function main() {
    const file = join(
        resolve(dirname(fileURLToPath(import.meta.url)), ".."),
        "apps/desktop/src/lib/features/thanks/thanks.json",
    );
    const current = JSON.parse(readFileSync(file, "utf8"));
    const { GITHUB_TOKEN, GITHUB_REPOSITORY, CROWDIN_PERSONAL_TOKEN, CROWDIN_PROJECT_ID } = process.env;

    const contributors = await attempt("GitHub contributors", () =>
        fetchContributors(GITHUB_REPOSITORY || DEFAULT_REPO, GITHUB_TOKEN),
    );
    const translators =
        CROWDIN_PERSONAL_TOKEN && CROWDIN_PROJECT_ID
            ? await attempt("Crowdin translators", () => fetchTranslators(CROWDIN_PROJECT_ID, CROWDIN_PERSONAL_TOKEN))
            : (console.warn("Crowdin secrets are not set; keeping the current translators."), null);

    if (contributors === null && translators === null) process.exitCode = 1;
    writeFileSync(file, JSON.stringify(mergeThanks(current, { contributors, translators }), null, 4) + "\n");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
