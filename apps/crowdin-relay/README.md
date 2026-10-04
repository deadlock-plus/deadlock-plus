# Crowdin relay

A small Cloudflare Worker. It receives Crowdin webhooks and posts them to a Discord channel as Components V2 messages.

Crowdin's webhook form cannot add `?with_components=true` to the Discord URL, and Discord rejects Components V2 without it. The Worker sits in between and adds it.

## What it posts

- **File events** (`file.*`): project, language, file path, and a link to the editor.
- **Suggestion events** (`suggestion.*`): project, language, key, source text, translation, author, reviewer (approved and disapproved only), and a link to the string.
- **Source string events** (`string.added`, `string.updated`, `string.deleted`): project, file path, key, source text, context, who made the change, and a link to the string.
- **Anything else**: a header with the event name and the project.

The Worker accepts both a single event and a batched `{ "events": [...] }` body. A batch posts one Discord message per event.
The Worker reads Crowdin's default payload. Do not set a custom payload in Crowdin.

## One-time setup

1. In Discord, create a channel webhook. Set its name and avatar there. The message sets `username` only.
2. Generate a path secret, for example `openssl rand -hex 24`.
3. From this folder, set the two Worker secrets:
    ```sh
    pnpm install
    pnpm wrangler secret put DISCORD_WEBHOOK_URL
    pnpm wrangler secret put PATH_SECRET
    ```
    The first deploy has to exist before `secret put` works: run `pnpm deploy` once, or let the deploy workflow do it.
4. Deploy: push a change under `apps/crowdin-relay/` to `main`, or run `pnpm deploy`.
5. In Crowdin, open the project's Tools > Webhooks and add a webhook:
    - URL: `https://crowdin-relay.<your-subdomain>.workers.dev/<PATH_SECRET>`
    - Method: `POST`, content type: `application/json`
    - Events: the file, source string and suggestion events you want
    - Batch webhooks: off
    - Custom payload: off

## Deploy workflow

`.github/workflows/deploy-crowdin-relay.yml` runs the tests, then `wrangler deploy`. It needs two repo secrets and skips while either is unset:

- `CLOUDFLARE_API_TOKEN`: an API token with the Workers edit scope.
- `CLOUDFLARE_ACCOUNT_ID`: the Cloudflare account id.

The Worker's own runtime secrets are set by hand (step 3). The workflow does not touch them.

## Rotate the path secret

1. `pnpm wrangler secret put PATH_SECRET` with a new value.
2. Update the webhook URL in Crowdin.

The old URL answers 404 as soon as the secret changes.

## Develop

- `pnpm test`, `pnpm typecheck`, `pnpm format:check`.
- `pnpm dev` runs the Worker locally. Put `DISCORD_WEBHOOK_URL` and `PATH_SECRET` in a `.dev.vars` file (gitignored).
- Sample payloads live in `test/fixtures/`. Event shapes follow Crowdin's webhook documentation.

## Responses

| Status | Meaning                                         |
| ------ | ----------------------------------------------- |
| 204    | Posted to Discord                               |
| 400    | Body is not a JSON object                       |
| 404    | Wrong path                                      |
| 405    | Not a `POST`                                    |
| 500    | `DISCORD_WEBHOOK_URL` or `PATH_SECRET` is unset |
| 502    | Discord rejected the message or was unreachable |
