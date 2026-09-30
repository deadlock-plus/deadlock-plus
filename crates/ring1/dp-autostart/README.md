# dp-autostart

Starts Deadlock+ at login on Linux and macOS. It is ring 1 because it writes to the OS's login-item locations. Windows autostart (Task Scheduler) stays in the app crate.

## Public API

- `state() -> Result<State, String>`, `enable()`, `disable()` exist on non-Windows targets only. `State` is `{ enabled, stale }`.
- `stale` means the entry launches a different exe than the one to launch now.
- `LAUNCH_ARG` is `--autostart`.
- `desktop_entry` (Linux) and `launch_agent` (macOS) render and parse the entry files. They compile on every host.
- `launch_exe`, `read_state`, `write_entry`, `remove_entry` are the file-level building blocks.

## Dependencies

- No external crates.
- No workspace crates.

## Platform behaviour

- Linux writes `$XDG_CONFIG_HOME/autostart/deadlock-plus.desktop`. A relative or empty `$XDG_CONFIG_HOME` is ignored and `~/.config` is used.
- macOS writes `~/Library/LaunchAgents/app.deadlockplus.autostart.plist` with `RunAtLoad`. It never calls `launchctl`, so the agent first runs at the next login.
- Both launch the exe with `--autostart`. When `$APPIMAGE` is set, that image file is the exe.
- Disabling deletes the file. A missing file is not an error.

## Gotchas

- The `Exec` value is escaped twice: reserved characters inside the quotes, then the desktop-file string escape. `%` becomes `%%`.
- Nothing here has run on a real Linux or macOS desktop. The file formats follow the XDG Desktop Entry and launchd.plist documentation.

## Testing

```
cargo test -p dp-autostart
```

The tests render and parse both file formats and use temp directories for the file operations.
