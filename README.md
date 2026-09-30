# Deadlock+

[![CI](https://github.com/deadlock-plus/deadlock-plus/actions/workflows/ci.yml/badge.svg)](https://github.com/deadlock-plus/deadlock-plus/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/deadlock-plus/deadlock-plus)](https://github.com/deadlock-plus/deadlock-plus/releases/latest)
[![Downloads (latest)](https://img.shields.io/github/downloads/deadlock-plus/deadlock-plus/latest/total?label=downloads%20%28latest%29)](https://github.com/deadlock-plus/deadlock-plus/releases/latest)
[![Downloads (total)](https://img.shields.io/github/downloads/deadlock-plus/deadlock-plus/total?label=downloads%20%28total%29)](https://github.com/deadlock-plus/deadlock-plus/releases)
[![License: GPL-3.0-or-later](https://img.shields.io/github/license/deadlock-plus/deadlock-plus)](LICENSE)
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

A companion app for [Deadlock](https://store.steampowered.com/app/1422450/). Windows is the main platform; macOS and Linux support is best-effort (see [Requirements](#requirements)). Built with Tauri v2, SvelteKit and Rust.

Unofficial fan tool. Not made by, affiliated with or endorsed by Valve Corporation.

## Features

- **Status bar**: bottom strip, right to left: whether Deadlock is running (checked every 5 seconds), then the Deadlock API ingest state.
- **Home**: the landing page. A time-of-day greeting with your Steam avatar, counts of blocked regions, muted players and saved replays, and shortcuts to every tool. A card shows when Steam's usual weekly maintenance is next due. Four live cards show your current rank with progress to the next subrank, ranked form over your last 20 matches, the latest patch or news item (with an unread count), and your last play session.
- **Server Picker**: block or unblock Steam Datagram Relay regions with Windows Firewall rules. Sort by region, ping or blocked state; the sort is remembered across pages and restarts.
  - Live ping per region, presets ("only allow these" or "block these"), and import of rules from ServerPickerX and CS2ServerPicker.
  - Regions with several relays for one city (Frankfurt, Stockholm, India) are merged.
  - Only rules this app created (`deadlock_plus_*`) are ever changed or removed.
- **Connection**: live server, ping and packet loss for your current match, with an optional ExitLag comparison.
- **Stats**: winrate, streaks, playtime and per-hero numbers (games, winrate, KDA, average souls) for ranked, unranked or all modes over 7, 30, 90 days or all time, worked out on your PC from the match history the Deadlock API holds for your account. Unranked matches carry no result in the API, so wins and losses there are worked out from the winning team. It only counts matches the API has seen. Each hero is one card with its winrate bar, stats and leaderboard bars, and the top of the page shows your last 12 results, most played hero and best winrate hero. Also shows progress toward leaderboard eligibility: the region board (75 games in 30 days, 500 total) and each hero board (30 games and 100 lifetime wins on the hero, 500 total). Approximate, since Valve does not publish which queues count.
- **Rank**: your current rank with a progress bar and how many straight wins reach the next rank, how many demotion shields you have left, winrate and net progress over your last 20 ranked matches, what your next win and next loss are worth, the winrate needed to hold your rank, and a climb forecast (days to the next subrank and tier at your recent pace), a progress graph (last 20, 50 or 100 matches, shield saves marked), your recent ranked matches and your promotions and demotions. From the Deadlock API.
- **Sessions**: your matches grouped into play sessions (a new one after 90 minutes idle), with games, wins and losses, length and rank change per session. Below that, plain-language findings compared with your overall winrate: results after two losses in a row, where in a session your results drop, switching heroes or staying after a loss, queueing straight back up or waiting, time of day, and weekdays against weekends. A difference is only reported when enough games back it up. Sessions get an edge colour and label (excellent, good, even or rough), and the top of the page repeats your last session if it went well, or your best recent one. An optional break reminder (off by default) appears after three losses in a row.
- **Mutes**: view, add, unmute, export and import your muted players. A timestamped backup is made before every change, and edits are blocked while the game runs.
- **Replays**: browse saved match replays with hero, KDA and result, see which are partial or from an older build, pin the ones to keep, and delete or clean up the rest (Recycle Bin or permanent).
- **Storage**: see what Deadlock and Deadlock+ use on disk, grouped by owner and marked as regenerating, history, backup, yours or managed by mods. Only regenerable files (shader cache, console log, this app's own mute backups) can be cleared here.
- **Performance**: two tools. **Frametimes** records how evenly Deadlock presents frames while you play (start, play, stop): average FPS, 1% and 0.1% lows, frametime percentiles, and a list of stutter spikes, with a live graph while recording. Time spent tabbed out is left out so it does not read as stutter. It listens to Windows' own graphics events, so nothing is injected into the game, and it needs Deadlock+ to run as administrator. **Addon scripts** scans the scripts inside your installed Deadlock addons for patterns that can hurt frametimes (timers that are cleared without being cancelled, or re-arm themselves with no cancel). Each addon shows as flagged, scanned with no findings, or having no scripts. It is a best-effort check on script text: a finding is a hint, and a clean result does not clear a mod. Addons appear as each one finishes, with a progress bar. A scan also runs once in the background shortly after launch (turn it off under Background work in Settings > Startup and background), and the status bar shows its progress and a "Potential performance issues found" link when something is flagged. Nothing is changed or run. **Compare** sets two saved frametime runs side by side (save a run with a label after recording, noting which addons were on) and can copy or save a plain-text report. A difference is consistent with a mod being the cause, not proof of it.
- **Settings**: a full-window overlay opened from the gear beside your account in the sidebar (Esc closes it), with a search box and a category list, like Discord. Categories: Appearance (theme, reduced motion, accessible font), Startup and background, Notifications, and Privacy (match data sharing), followed by version details you can copy for bug reports, app updates (check now, and a switch for the check on launch; when an update is found or downloading, a download button appears in the titlebar beside the window controls and opens this section), a What's new list built from the changelog (the same notes pop up once after an update), a Diagnostics section (this session's log, with copy, save and open-folder buttons for bug reports, and an Expand button that grows the viewer to fill the settings window) and the licences for everything the app uses. Startup, background and notification options:
  - Start with Windows, through a Task Scheduler task so there is no UAC prompt at sign-in. The app opens hidden in the tray.
  - Keep running in the tray when the window is closed. The tray icon is always present; left-click or "Show Deadlock+" reopens the window, "Quit" exits. Launching a second copy shows the running one.
  - Background work: the addon scan and patch note indexing run in the background and pause while Deadlock is running, then carry on when you close it. The status bar shows a Run now button to override that, and a scan you start yourself runs right away. One switch allows background work at all, another turns the pausing off, and each task has its own switch and a choice of keep running, pause or slow down while Deadlock runs. Turning patch note indexing off stops the background work; search still covers what is already indexed.
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
- **First-run welcome**: three steps on the first launch (game found, why the UAC prompt and firewall rules, optional extras, all off by default). Skippable.
- **Match data sharing** (opt-in: asked on first launch, changeable in Settings): reads Deadlock replay links from Steam's local HTTP cache and uploads the match IDs, replay salts and your Steam account ID to the Deadlock API so the community database can fetch those matches. Nothing else is read or sent. Off until you say yes.

## Requirements

Windows 10 or 11 and WebView2. The app asks for administrator rights on launch because Windows Firewall rules and network event tracing need them.

**macOS and Linux:** support is offered on a best-effort basis and is mostly untested. The Server Picker is expected to work (blocking asks for your password) but is untested. The Connection page is expected to work too (it asks for your password when you press Allow) but is untested. Frametimes works on Linux only, through a Vulkan layer you install from the Performance page and switch on with a Steam launch option (untested; it is not available on macOS or under Wine). Autostart, Steam and Deadlock discovery under Wine, Proton and Whisky, and the Linux AppImage and deb and macOS dmg installers are new and untested too. If you run into a problem on macOS or Linux, please [open an issue](https://github.com/deadlock-plus/deadlock-plus/issues). A fix or a pull request that solves it is a big plus.

The installer is not yet Windows code-signed, so SmartScreen may warn on first run (More info, Run anyway). Updates are still verified: every release is signed with the project's update key and the app rejects anything that does not match.

## Development

Setup, checks, conventions and the release process are in [CONTRIBUTING.md](CONTRIBUTING.md).

## Licences and attribution

Deadlock+ is licensed under [GPL-3.0-or-later](LICENSE). See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for the bundled fonts and artwork. The Deadlock fonts remain their owners' property and will be removed on request.
