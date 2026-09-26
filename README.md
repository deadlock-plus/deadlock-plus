# Deadlock+

A companion app for [Deadlock](https://store.steampowered.com/app/1422450/). Windows only for now; macOS and Linux support is planned. Built with Tauri v2, SvelteKit and Rust.

Unofficial fan tool. Not made by, affiliated with or endorsed by Valve Corporation.

## Features

- **Home**: the landing page. A time-of-day greeting with your Steam avatar, whether Deadlock is running, counts of blocked regions, muted players and saved replays, and shortcuts to every tool. A card shows when Steam's usual weekly maintenance is next due. Four live cards show your current rank with progress to the next subrank, ranked form over your last 20 matches, the latest patch or news item (with an unread count), and your last play session.
- **Server Picker**: block or unblock Steam Datagram Relay regions with Windows Firewall rules.
  - Live ping per region, presets ("only allow these" or "block these"), and import of rules from ServerPickerX and CS2ServerPicker.
  - Regions with several relays for one city (Frankfurt, Stockholm, India) are merged.
  - Only rules this app created (`deadlock_plus_*`) are ever changed or removed.
- **Connection**: live server, ping and packet loss for your current match, with an optional ExitLag comparison.
- **Stats**: winrate, streaks, playtime and per-hero numbers (games, winrate, KDA, average souls) for ranked, unranked or all modes over 7, 30, 90 days or all time, worked out on your PC from the match history the Deadlock API holds for your account. Unranked matches carry no result in the API, so wins and losses there are worked out from the winning team. It only counts matches the API has seen. Each hero is one card with its winrate bar, stats and leaderboard bars, and the top of the page shows your last 12 results, most played hero and best winrate hero. Also shows progress toward leaderboard eligibility: the region board (75 games in 30 days, 500 total) and each hero board (30 games and 100 lifetime wins on the hero, 500 total). Approximate, since Valve does not publish which queues count.
- **Rank**: your current rank with a progress bar and how many straight wins reach the next rank, how many demotion shields you have left, winrate and net progress over your last 20 ranked matches, what your next win and next loss are worth, the winrate needed to hold your rank, and a climb forecast (days to the next subrank and tier at your recent pace), a progress graph (last 20, 50 or 100 matches, shield saves marked), your recent ranked matches and your promotions and demotions. From the Deadlock API.
- **Sessions**: your matches grouped into play sessions (a new one after 90 minutes idle), with games, wins and losses, length and rank change per session. Below that, plain-language findings compared with your overall winrate: results after two losses in a row, where in a session your results drop, switching heroes or staying after a loss, queueing straight back up or waiting, time of day, and weekdays against weekends. A difference is only reported when enough games back it up. Sessions get an edge colour and label (excellent, good, even or rough), and the top of the page repeats your last session if it went well, or your best recent one. An optional break reminder (off by default) appears after three losses in a row.
- **Mutes**: view, add, unmute, export and import your muted players. A timestamped backup is made before every change, and edits are blocked while the game runs.
- **Replays**: browse saved match replays with hero, KDA and result, see which are partial or from an older build, pin the ones to keep, and delete or clean up the rest (Recycle Bin or permanent).
- **Storage**: see what Deadlock and Deadlock+ use on disk. Only regenerable files (shader cache, console log, this app's own mute backups) can be cleared here.
- **Settings**: a full-window overlay opened from the gear beside your account in the sidebar (Esc closes it), with a search box and a category list, like Discord. Categories: Appearance (theme, reduced motion, accessible font), Startup and background, Notifications, and Privacy (match data sharing), followed by version details you can copy for bug reports, app updates (check now, and a switch for the check on launch; when an update is found or downloading, a download button appears in the titlebar beside the window controls and opens this section), a What's new list built from the changelog (the same notes pop up once after an update), a Diagnostics section (this session's log, with copy, save and open-folder buttons for bug reports, and an Expand button that grows the viewer to fill the settings window) and the licences for everything the app uses. Startup, background and notification options:
  - Start with Windows, through a Task Scheduler task so there is no UAC prompt at sign-in. The app opens hidden in the tray.
  - Keep running in the tray when the window is closed. The tray icon is always present; left-click or "Show Deadlock+" reopens the window, "Quit" exits. Launching a second copy shows the running one.
  - Steam maintenance reminder: a Windows notification before the weekly maintenance. Steam publishes no schedule, so the default is the usual slot (Wednesday 00:00 UTC, 15 to 30 minutes) and you can change the day, time and lead time. It needs the app running.
  - Patch and news alerts (off by default): a Windows notification when a new patch note or Steam announcement appears in the Deadlock API's patch feed, plus an Updates page listing them as cards with the update type (Minor Update, Matchmaking Update, ...), a bulleted text preview and Steam's banner art (a placeholder when the post has none). A forum post and the Steam post it mirrors show as one card. The first check lists what already exists as read, without notifying, and opening the Updates page fetches the feed once. It needs the app running.

## Logs

Written to the app's log folder (`%LOCALAPPDATA%\app.deadlockplus\logs` on Windows; Settings > Diagnostics has an Open folder button, a level filter, search and an Auto-scroll switch (on by default), and it starts at INFO, also when expanded):

- Line format: `[HH:mm:ss] [thread | LEVEL] [logger]: message`. Background threads are named (`ingest-watcher`, `etw-session`, `etw-processor`, `network-aggregator`, `network-sampler`).
- `latest.log` (info and above), `debug.log` (debug and above), `trace.log` (everything).
- Each file rolls into `yyyy-MM-dd-i.log.gz` on startup and when the day changes; the newest 60 archives are kept.
- The web view's `console.*` output, uncaught errors and panics are written to the same files. Steam IDs and IPs are not logged; copying or saving from Diagnostics hides user names in paths.

## Privacy and network use

Everything runs locally except:

- Steam's public SDR relay list (server picker).
- Reminders and the tray are local. The maintenance reminder uses the clock only and sends nothing.
- The public [Deadlock API](https://api.deadlock-api.com): player name search, mute list names, match details for your replays, your match history, your current rank and the rank names (when you open the Stats, Rank or Sessions page; your Steam account id is in that request), and the patch feed (when you open the Updates page, and every 15 minutes if patch and news alerts are on). The only thing about you it receives is your Steam account id in the match history request.
- Steam's public CDN, for the banner images on the Updates page.
- GitHub Releases, for the app's own update check (on launch, unless turned off in Settings > Version, and when you press Check now). It sends no account data. Updates only install when you click.
- **Match data sharing** (opt-in: asked on first launch, changeable in Settings): reads Deadlock replay links from Steam's local HTTP cache and uploads the match IDs, replay salts and your Steam account ID to the Deadlock API so the community database can fetch those matches. Nothing else is read or sent. Off until you say yes.

## Requirements

Windows 10 or 11 and WebView2. macOS and Linux are not supported yet, but both are planned. The app asks for administrator rights on launch because Windows Firewall rules and network event tracing need them.

The installer is not yet Windows code-signed, so SmartScreen may warn on first run (More info, Run anyway). Updates are still verified: every release is signed with the project's update key and the app rejects anything that does not match.

## Development

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

### Security notes

- The web view runs under a Content Security Policy (`app.security` in `src-tauri/tauri.conf.json`): scripts from the app only, network calls to the Deadlock API only, images over https. Add an origin there before a new feature fetches from it.
- Saving a file goes through the backend (`save_text_file`), which opens the save dialog itself. The web view never supplies a path.
- Firewall block requests are validated in `features/server_picker/validate.rs` (group id characters, public unicast relay IPs only).

### Adding a feature

1. Rust: `src-tauri/src/features/<name>/mod.rs`; export it in `features/mod.rs` and register its commands in `lib.rs`.
2. Frontend: `src/lib/features/<name>/` and a route in `src/routes/<name>/+page.svelte`.
3. Add an entry to `src/lib/features/registry.ts`. It drives both the sidebar and the tool cards on Home, and its array order is the sidebar order (Server Picker, Connection, Stats, Rank, Sessions, Updates, Mutes, Replays, Storage; Home is fixed in the sidebar itself, and the Settings gear opens the overlay).

Adding a game to the server picker is data only: add an entry to `src-tauri/resources/games.json`.

### Releasing

1. Bump `version` in `package.json`, run `pnpm version:sync`, and move the `[Unreleased]` notes in `CHANGELOG.md` under a `## [x.y.z] - date` heading. The release fails without that entry; its text is the release body and the "What's new" dialog.
2. Tag `vx.y.z` and push. `.github/workflows/release.yml` builds, signs the update, and creates a **draft** release with the installer and `latest.json`. The app reads `latest.json` from the newest published release, so publishing the draft is what ships the update.
3. Repo secrets: `TAURI_SIGNING_PRIVATE_KEY` (contents of the key from `pnpm tauri signer generate`) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Losing the key means installed copies can never update; keep a backup outside the repo. To build locally, set the same two variables in the shell.
4. Optional Windows code signing through SignPath: create the SignPath project (slug `deadlock-plus`, policy `release-signing`), add the secret `SIGNPATH_API_TOKEN` and the variable `SIGNPATH_ORGANIZATION_ID`. The workflow skips the step while the variable is unset.

### Dependency licences

`node scripts/gen-licenses.mjs` regenerates `src/lib/generated/dependency-licenses.json`, which Settings displays. It needs `cargo install cargo-about --locked --features cli`. Rerun it after changing dependencies.

## Licences and attribution

Deadlock+ is licensed under [GPL-3.0-or-later](LICENSE). See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for the bundled fonts and artwork. The Deadlock fonts remain their owners' property and will be removed on request.
