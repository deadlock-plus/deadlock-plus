# dp-game

Detects whether Deadlock is running. It is ring 1 because it reads the OS process list.

## Public API

- `PROCESS_NAME` is `"deadlock.exe"`.
- `is_process(name)` compares an `OsStr` with `PROCESS_NAME`, ignoring ASCII case.
- `is_running()` refreshes the process list and returns whether any process matches.

## Dependencies

- `sysinfo`
- No workspace crates.

No platform-specific code. `sysinfo` handles every host.

## Gotchas

- The match is on the exact name `deadlock.exe` and nothing else. Deadlock runs under that name in Wine and Proton, so Linux hosts use the same check.
- `is_running` builds a fresh `System` on each call and reads only process names. It is not a cached view, so avoid calling it in a tight loop.

## Testing

```
cargo test -p dp-game
```

The tests cover name matching only. They do not look for a real process.
