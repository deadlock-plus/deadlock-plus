# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
