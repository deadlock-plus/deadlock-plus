# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
