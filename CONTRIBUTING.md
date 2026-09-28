# Contributing

Deadlock+ is a Tauri v2 app: a Rust backend and a SvelteKit (Svelte 5) frontend. Windows only for now.

## Setup

Needs Node with pnpm, and Rust.

```
pnpm install
pnpm tauri dev            # prompts for UAC
pnpm check                # svelte-check
pnpm format               # Prettier (4 spaces, width 120); CI runs pnpm format:check
pnpm test                 # Vitest
cargo test --lib --manifest-path src-tauri/Cargo.toml   # also regenerates src/lib/generated/types
cargo fmt -- --config max_width=120,use_small_heuristics=Max   # run in src-tauri
pnpm tauri build          # NSIS installer (per-machine, branded images); needs the update signing key, see Releasing
```

CI also runs `pnpm audit --prod` and `cargo audit`, and fails if `src/lib/generated/types` is out of date. Commit the regenerated files with any Rust type change. `.editorconfig` sets a 4-space indent (2 for `package.json` and YAML).

## Security notes

- The web view runs under a Content Security Policy (`app.security` in `src-tauri/tauri.conf.json`): scripts from the app only, network calls to the Deadlock API only, images over https. Add an origin there before a new feature fetches from it.
- Saving a file goes through the backend (`save_text_file`), which opens the save dialog itself. The web view never supplies a path.
- Firewall block requests are validated in `features/server_picker/validate.rs` (group id characters, public unicast relay IPs only).

## Conventions

- The web view never supplies paths for reads, writes or deletes. Rust commands derive or validate them (fixed allowlists, checks that a path stays inside its root).
- The web view does not write files. Persisted data goes through backend commands: `features/kv.rs` (stores must be listed in `STORES`), `features/versioned.rs` (`{ schema_version, data }` envelope; every stored file needs a `MIGRATIONS` slice, and a file with a newer version is refused, never overwritten), `features/atomic.rs` (`write_atomic`) and `features/export.rs`. Changing the stored fields of the alerts file means bumping `STORE_VERSION` in `alerts/mod.rs`.
- New setting: add an item to `settings/catalog.ts`, then wrap it in `{#if show(id)}` in its section component.
- User-facing names are "Mutes" and "Replays"; code, routes, files and types keep `voice-ban` and `demo`.
- Firewall rules are named `deadlock_plus_<id>_tcp` and `_udp`; the prefix stays. The bundle identifier `app.deadlockplus` names the app data folder, so changing it orphans users' data.
- `tauri.macos.conf.json` replaces arrays instead of merging, so it repeats the whole window entry. Keep it in sync with `tauri.conf.json`.
- Tailwind's `@theme inline` inlines values. To make a token overridable at runtime, point it at a `:root` variable.
- Never read ExitLag's `user_*` or token rows, and never commit ISP or account data.
- The consent text for uploading match salts to the Deadlock API is worded deliberately; do not change it without discussion.
- Moving the repo breaks `node_modules` and `src-tauri/target` (absolute paths). Reinstall and rebuild.
- Icons: put the source at `src-tauri/icons/source-1024.png`, run `pnpm tauri icon`, then delete the generated `android/`, `ios/` and `64x64.png`.
- Font notices live in both `THIRD-PARTY-NOTICES.md` and `src/lib/features/settings/licenses.ts`; keep them in sync.

## Adding a feature

1. Rust: `src-tauri/src/features/<name>/mod.rs`; export it in `features/mod.rs` and register its commands in `lib.rs`.
2. Frontend: `src/lib/features/<name>/` and a route in `src/routes/<name>/+page.svelte`.
3. Add an entry to `src/lib/features/registry.ts`. It drives both the sidebar and the tool cards on Home, and its array order is the sidebar order (Server Picker, Connection, Stats, Rank, Sessions, Updates, Mutes, Replays, Storage; Home is fixed in the sidebar itself, and the Settings gear opens the overlay).

Adding a game to the server picker is data only: add an entry to `src-tauri/resources/games.json`.

## Releasing

1. Bump `version` in `package.json`, run `pnpm version:sync`, and move the `[Unreleased]` notes in `CHANGELOG.md` under a `## [x.y.z] - date` heading. The release fails without that entry; its text is the release body and the "What's new" dialog.
2. Tag `vx.y.z` and push. `.github/workflows/release.yml` builds, signs the update, and creates a **draft** release with the installer and `latest.json`. The app reads `latest.json` from the newest published release, so publishing the draft is what ships the update.
3. Repo secrets: `TAURI_SIGNING_PRIVATE_KEY` (contents of the key from `pnpm tauri signer generate`) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Losing the key means installed copies can never update; keep a backup outside the repo. To build locally, set the same two variables in the shell.
4. Optional Windows code signing through SignPath: create the SignPath project (slug `deadlock-plus`, policy `release-signing`), add the secret `SIGNPATH_API_TOKEN` and the variable `SIGNPATH_ORGANIZATION_ID`. The workflow skips the step while the variable is unset.

## Dependency licences

`node scripts/gen-licenses.mjs` regenerates `src/lib/generated/dependency-licenses.json`, which Settings displays. It needs `cargo install cargo-about --locked --features cli`. Rerun it after changing dependencies.
