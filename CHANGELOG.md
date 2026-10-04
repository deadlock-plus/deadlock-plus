# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-10-04

### Added

- The sidebar has a Discord button that opens the Deadlock+ server.

#### Command palette

- Press Ctrl+K (Cmd+K on macOS) to open a command palette.
- It jumps to any page or Settings section, and runs actions like checking for updates.
- It lists every keyboard shortcut.

#### Page shortcuts

- Press Ctrl+1 to Ctrl+9 (Cmd on macOS) to jump to the first nine pages in the sidebar.

#### Language setting

- Settings has a new Language section. It follows your system language by default.
- Each language shows its flag and how much of the app is translated, and you pick one from the list.
- All app text, error messages and notifications now come from a translation catalog.
- The tray menu and desktop notifications follow the chosen language.
- Translations are community-made on Crowdin, and more languages will arrive as they are approved.

### Changed

- Theme and delete-method choices now move with the arrow keys.
- Server Picker keeps keyboard focus on the block switch while it updates.
- Notifications can be reached with the arrow keys.
- Many errors and loading messages are now announced by screen readers.

## [0.6.1] - 2026-10-03

### Added

- The sidebar has a "Support Deadlock+" button that opens the Ko-fi page, in and outside Settings.
- The support button's logo pulses now and then while Deadlock+ is in view.
- A one-time toast on your third launch points to the support page.

## [0.6.0] - 2026-10-03

### Added

- Home has a new Deadlock tile that launches the game through Steam and shows when it is running.

#### Instant match results

- Instant match results is a new opt-in setting.
- It is Windows only and off by default.
- Finished matches show in Stats and Sessions right away, before the Deadlock API has them.
- Deadlock+ reads them from the running game on this PC and never uploads them.
- These matches carry a "Syncing" badge until the Deadlock API confirms them.
- A just-finished ranked match moves your rank estimate right away, marked as estimated.
- Setup offers this option.

#### Recover missing match salts through Steam

- Recovering missing match salts through Steam is a new opt-in setting.
- It is off by default.
- It signs in to Steam with your saved login and asks Deadlock's game servers for matches the community database lacks.
- It never runs while Deadlock is open.
- It is limited to 40 matches per account per day.
- Your Steam login stays on this PC.
- Setup offers this option.

### Changed

- Steam update links now open in the Steam app when it is installed.
- They fall back to your browser otherwise.

## [0.5.0] - 2026-09-30

### Added

- A step-by-step setup page. You see it once after updating, and Esc or Skip ends it.
- What's New is now a full page. It opens after an update and from Settings > About.
- A Deadlock API status item in the status bar. It shows online, degraded or offline.
- A crash report dialog on the next launch after an error or an unclean exit.
- The report is a plain-text file with the last 500 log lines. User names and tokens are masked.
- The dialog opens the report folder or a pre-filled GitHub issue.
- Nothing is sent anywhere. You choose whether to share the report.
- Early macOS and Linux support. It is best-effort and mostly untested.
- The Connection page works on macOS and Linux. It asks for your password when you press Allow.
- Server Picker blocking works on Linux (nftables) and macOS (pf). It asks for your password to apply or remove a block.
- Frame capture works on Linux through a Vulkan layer. Install it from the Performance page.
- A Steam launch option turns the frame layer on for Deadlock only.
- The Linux packages include the frame layer.
- Deadlock and Steam are found on Linux (native, Flatpak, snap) and under Wine, Proton, Whisky and CrossOver.
- Deadlock is found in extra Steam libraries inside Wine prefixes.
- Start with login works on Linux and macOS.
- The startup switch is named for your system: Start with Windows, Start with macOS or Start at login.
- The welcome tour lists the features missing on macOS and Linux, and where to report problems.
- Linux AppImage and deb builds, and macOS dmg builds for Apple silicon. They are unsigned and untested.

### Changed

- The data-sharing choice is now part of setup instead of a popup. If you skip it, sharing stays off.
- Every page uses the same content width and header size.
- Exported logs also mask tokens and /home names.
- Folders and files open through the system file manager on every platform.
- On macOS and Linux, replays go to the Trash instead of the Recycle Bin.
- The Connection page hides ExitLag on macOS and Linux.
- Version details show the right account and web view names on macOS and Linux.

### Fixed

- Patch note search no longer fails when a patch repeats the same line.
- Dialogs no longer dim the title bar, only the page content.
- The sidebar toggle, window controls and notification centre work while a dialog is open.

## [0.4.1] - 2026-09-29

### Added

- Server blocks update themselves when Valve changes its relay addresses. The block stays on while it updates.
- The check runs after launch, every 30 minutes and when you open the server picker.
- A notification tells you when a block was updated, or needs re-applying.
- Background work: an "Updating server blocks" task you can turn off.

### Fixed

- Background work settings (the pause switch, per-task choices and switches) now save. They reset every time Deadlock+ restarted.

## [0.4.0] - 2026-09-29

### Added

- Performance page with an addon script scan. It flags patterns that can hurt frametimes, such as timers that pile up. Results are hints, not proof.
- Frametimes: record Deadlock's frame pacing while you play. Shows average FPS, 1% and 0.1% lows, percentiles and stutter spikes. Time spent tabbed out is excluded.
- Save a frametime run with a label, then compare two runs side by side.
- Patch note indexing and addon scans pause while Deadlock runs and resume when you close it.
- Background work setting: allow background work, pause while Deadlock runs, and per job (addon scan, patch note indexing) run by itself and choose keep running, pause or slow down while Deadlock runs.
- Storage rows show a badge (regenerates, history, backup, yours or managed by mods), what clearing does, item count, age and a size bar.
- Storage lists saved frametime runs with their size and a link to Performance.
- Storage: Show paths toggle and a Clear regenerable button for the shader cache and console log.
- The status bar shows "paused while Deadlock runs" with a Run now button. A scan you start yourself runs right away.
- Copy or save a comparison as a text report.

### Changed

- Storage is grouped by owner: Deadlock, Deadlock+, mods and backups. Rows are sorted by size.
- Storage splits "Deadlock+ data" into Settings, Server presets, Replay pins and cleanup rules, Connection history, Alerts and notifications, Patch notes index, Stats cache, Server list cache, Replay info cache and Other app files.
- Storage: "Deadlock+ logs" is now "App logs".
- Home: the Tools list is centered, so a short last row sits in the middle.

## [0.3.0] - 2026-09-28

### Added

- Deadlock+ now checks for updates periodically while it's open, not just at launch, so a release doesn't go unnoticed during a long session.
- A notification bell in the title bar collects patch/news alerts and maintenance reminders in one place.
- Notifications can be marked read individually or all at once.
- The tray and taskbar icons show a green badge when an update is available.
- The tray and taskbar icons show a red badge with an unread notification count.

### Fixed

- Patch notes: images now show at the same spot they appear in the real post, instead of always at the top, and can be clicked to view full-size.
- Patch notes: a bold section heading like "General" no longer shows up as literal bolded text (`**[ General ]**`).
- The maintenance reminder no longer fires again if the app restarts while already inside the lead window.
- Stats: a leaderboard-tracked hero with no games in the current filter now shows its lifetime game count instead of just "No games in this selection."

## [0.2.0] - 2026-09-28

### Added

- Search patch notes from the Updates page, going all the way back to October 2024 — find a change even if you only remember roughly what it said, not the exact wording.
- A big, full-width search bar on the Updates page.
- Search results show whether an update came from Steam or the forums.
- Search waits for patch notes to finish indexing in the background before showing results, so you don't get an incomplete list.
- View full patch notes for an update, with images where available. Forum-only updates without a full body show the forum's short preview instead, with a link to read the rest there.
- Two new themes: Nightshift (teal cyberpunk noir with amber trim) and Deadlock API (OLED black with electric red glow).
- Status bar shows progress while new patch notes are being indexed, including which one is currently being processed.
- Storage page: clear Deadlock+'s own log files with a new `Logs` entry.

### Fixed

- Status bar items are now separated by a dot when several show at once.
- The main window no longer sits empty for a couple of seconds after launching.
- Some updates in the Updates list showed the wrong date and sorted out of order.

## [0.1.2] - 2026-09-26

### Added

- First-run welcome: finds the game, explains the administrator prompt and firewall use, and offers the optional extras (start with Windows, tray, alerts, maintenance reminder).
- Status bar shows whether Deadlock is running.

### Changed

- The Server Picker remembers its sort column and direction across page changes and restarts.
- Home no longer shows the "Deadlock is running" line; the status bar has it.

### Fixed

- Expanding or collapsing the Diagnostics log viewer no longer moves focus to the first settings category and pops up its tooltip.

## [0.1.1] - 2026-09-26

### Added

- Server Picker columns sort on click: Region, Direct ping and Blocked. Click again to reverse. Default is Region A-Z.
- Server Picker shows a warning banner while Deadlock is running: restart the game after changing blocks.
- New installs start with six Server Picker presets: Asia, EU All, EU West, USA, USA East and ZA. They are ordinary presets: edit or delete them, and they do not come back.

## [0.1.0] - 2026-09-26

First release. Windows only.

### Added

- Home page with a greeting, quick counts and live cards for your rank, recent form, latest update and last session.
- Server Picker: block or unblock Steam Datagram Relay regions with Windows Firewall rules, with live ping, presets, and import from ServerPickerX and CS2ServerPicker.
- Connection page: live server, ping and packet loss for your current match, with an optional ExitLag comparison. History is kept between launches.
- Stats: winrate, streaks, playtime and per-hero numbers for ranked, unranked or all modes, plus progress toward leaderboard eligibility.
- Rank: progress to the next subrank, demotion shields, what your next win or loss is worth, break-even winrate, climb forecast and a progress graph.
- Sessions: play sessions with plain-language findings, session verdicts and an optional break reminder.
- Mutes: view, add, unmute, export and import muted players, with a backup before every change.
- Replays: browse saved replays, pin the ones to keep, and clean up the rest.
- Storage: see what Deadlock and Deadlock+ use on disk and clear regenerable files.
- Updates page and optional Windows notifications for new patch notes and Steam announcements.
- Steam maintenance reminder with a configurable day, time and lead.
- Start with Windows through Task Scheduler (no UAC prompt at sign-in), and a tray icon with an optional keep-running-in-tray mode.
- Settings overlay with search: four themes, reduced motion, an accessible font option, diagnostics with a log viewer, and licences.
- Automatic update checks with signed updates and a "What's new" dialog. A titlebar button shows when an update is available.
- Opt-in match data sharing with the Deadlock API, asked on first launch.
- Offline banner. Stats pages show the last saved copy when the network is down.
- Window size and position are remembered.
