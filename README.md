# Deadlock+

[![CI](https://github.com/deadlock-plus/deadlock-plus/actions/workflows/ci.yml/badge.svg)](https://github.com/deadlock-plus/deadlock-plus/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/deadlock-plus/deadlock-plus)](https://github.com/deadlock-plus/deadlock-plus/releases/latest)
[![Downloads (latest)](https://img.shields.io/github/downloads/deadlock-plus/deadlock-plus/latest/total?label=downloads%20%28latest%29)](https://github.com/deadlock-plus/deadlock-plus/releases/latest)
[![Downloads (total)](https://img.shields.io/github/downloads/deadlock-plus/deadlock-plus/total?label=downloads%20%28total%29)](https://github.com/deadlock-plus/deadlock-plus/releases)
[![License: GPL-3.0-or-later](https://img.shields.io/github/license/deadlock-plus/deadlock-plus)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-join-5865F2?logo=discord&logoColor=white)](https://discord.gg/652zsGZZgD)
![Platform: Windows](https://img.shields.io/badge/platform-Windows-0078D4)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8DB)
![SvelteKit](https://img.shields.io/badge/SvelteKit-FF3E00?logo=svelte&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)

> [!IMPORTANT]
> ## 🛡️ Is this malware? Am I safe to download this?
>
> **You don't have to take our word for it. Check it yourself.**
>
> - **All the code is open.** Every line of the app is in this repo. Read it, search it, build it yourself.
> - **The installer is built in public.** It is made by [GitHub Actions workflows](.github/workflows/) that live in this repo. Nothing is built on a private machine.
> - **The build logs are public too.** Every release build has a full log on the [Actions tab](../../actions). You can see each step that turned the source into the installer you download.
> - **No hidden steps.** What you download is what the workflow built from the code you can read.
>
> 👉 **[Download the latest release](../../releases/latest)**

> [!WARNING]
> ## ⚠️ "Windows protected your PC" / "Windows says this file could be a risk"
>
> **This is Microsoft SmartScreen. It does not mean the file is malicious.**
>
> - SmartScreen flags any app it has not seen many times before. New and small apps trigger it, whatever the code does.
> - A code-signing certificate can remove the warning early, but they cost money each year. A new free tool like this one has to earn SmartScreen's trust through downloads over time.
> - The warning says "unknown publisher". It does not say "malware".
>
> To install anyway:
>
> 1. On the blue "Windows protected your PC" screen, click **More info**.
> 2. Click **Run anyway**.
>
> If you would rather not trust a warning screen or our word, read the code and the build logs above.

A companion app for [Deadlock](https://store.steampowered.com/app/1422450/), built with Tauri v2, SvelteKit and Rust. Windows is the main platform. macOS and Linux support is best-effort (see [Requirements](#requirements)).

Unofficial fan tool. Not made by, affiliated with or endorsed by Valve Corporation.

## Features

- **Server Picker.** Block or unblock Steam relay regions with Windows Firewall rules. Shows live ping and imports rules from ServerPickerX and CS2ServerPicker.
- **Connection.** Live server, ping and packet loss for your current match.
- **Stats.** Winrate, streaks, playtime and per-hero numbers over 7, 30, 90 days or all time, plus leaderboard eligibility progress.
- **Rank.** Progress to the next rank, demotion shields, what your next win or loss is worth, and a climb forecast.
- **Sessions.** Your matches grouped into play sessions, with plain-language findings about when you play best. Optional break reminder after three losses.
- **Mutes.** View, add, unmute, export and import muted players, with a backup before every change.
- **Replays.** Browse saved replays, pin the ones to keep and clean up the rest.
- **Storage.** See what Deadlock and Deadlock+ use on disk and clear regenerable files.
- **Performance.** Record frametimes (FPS, 1% lows, stutter spikes), scan addon scripts for patterns that hurt frametimes, and compare two runs.
- **Background options.** Start with Windows, run in the tray, get Steam maintenance reminders, and opt in to patch and news alerts.

Settings has theme options, a language picker, update checks, a What's new list and a Diagnostics log viewer for bug reports.
## Privacy and network use

Everything runs locally except the following.

- Steam's public SDR relay list, for the Server Picker.
- The public [Deadlock API](https://api.deadlock-api.com), for player name search, mute list names, replay match details, your match history, your rank and the rank names. The Stats, Rank and Sessions pages send your Steam account ID in the match history request, and that is the only thing about you the API receives. The patch feed loads when you open the Updates page, and every 15 minutes if patch and news alerts are on.
- Steam's public CDN, for banner images on the Updates page.
- GitHub Releases, for the app's update check. It runs on launch unless you turn it off in Settings > Version, and when you press Check now. It sends no account data, and updates install only when you click.
- Match data sharing (opt-in, asked on first launch, changeable in Settings). It reads Deadlock replay links from Steam's local HTTP cache and uploads the match IDs, replay salts and your Steam account ID to the Deadlock API, so the community database can fetch those matches. Nothing else is read or sent.
- Salt recovery through Steam (opt-in, off by default). It reads your saved Steam login on this PC, signs in as you and asks Deadlock's game servers for the salts of matches the community database is missing. Your login never leaves your PC. It never runs while Deadlock is open and is limited to 40 matches per account per day.

The maintenance reminder and tray use only the local clock and send nothing. The first-run welcome is three skippable steps covering game detection, why the UAC prompt and firewall rules are needed, and optional extras (all off by default).

## Requirements

Windows 10 or 11 and WebView2. The app asks for administrator rights on launch because Windows Firewall rules and network event tracing need them.

The installer is not yet Windows code-signed, so SmartScreen may warn on first run. Updates are still verified. Every release is signed with the project's update key, and the app rejects anything that does not match.

### macOS and Linux

Support is best-effort and mostly untested.

- The Server Picker and Connection page should work. Both ask for your password.
- Frametimes works on Linux only. It uses a Vulkan layer you install from the Performance page and enable with a Steam launch option. It is unavailable on macOS and under Wine.
- Autostart, Steam and Deadlock discovery under Wine, Proton and Whisky, and the AppImage, deb and dmg installers are new and untested.

If you hit a problem, please [open an issue](https://github.com/deadlock-plus/deadlock-plus/issues). A pull request with a fix is welcome.

## Translations

Deadlock+ is translated through [Crowdin](https://crowdin.com/project/deadlock-plus). English is the source language.

- Translate in the browser on the [Crowdin project](https://crowdin.com/project/deadlock-plus). You do not need to touch the code.
- Approved translations arrive as a pull request, and ship in the next release.
- Do not edit `locales/<lang>.json` by hand. Crowdin overwrites it.
- To fix English text, change `locales/en.json` in a pull request.
- Want a language that is not listed? [Open an issue](https://github.com/deadlock-plus/deadlock-plus/issues).

How the sync works is in [CONTRIBUTING.md](CONTRIBUTING.md#translations).

## Development

Setup, checks, conventions and the release process are in [CONTRIBUTING.md](CONTRIBUTING.md).

## Licences and attribution

Deadlock+ is licensed under [GPL-3.0-or-later](LICENSE). [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) covers the bundled fonts and artwork. The Deadlock fonts remain their owners' property and will be removed on request.
