# dp-game

Detects whether Deadlock is running. It is ring 1 because it reads the OS process list.

## Public API

- `PROCESS_NAME` is `"deadlock.exe"`.
- `is_process(name)` compares an `OsStr` with `PROCESS_NAME`, ignoring ASCII case.
- `MAX_AGE` is the shared process scan's maximum age (two seconds).
- `is_running()` returns whether any process matches the game.
- `start_time()` returns the oldest matching game's start time in Unix seconds, or `None`.
- `find_pid(matches)` returns the first matching process's PID, or zero.

## Dependencies

- `sysinfo`
- No workspace crates.

`sysinfo` reads the process list on every host. On Linux, the scan also reads the first command-line argument and excludes secondary threads.

## Gotchas

- Windows matches the exact process name `deadlock.exe`, ignoring ASCII case.
- On Linux, a process may also match by the basename of `argv[0]`, with either `/` or `\` as a separator. This covers Proton processes named `MainThrd` without accepting launchers that mention the game only in later arguments.
- `find_pid` applies its predicate to the process name and, on Linux, the `argv[0]` basename.
- Every query shares a scan cached for at most `MAX_AGE`. The first query scans immediately; the next query after expiry refreshes it.
- Only `argv[0]` is retained in the Linux cache; the rest of the command line is discarded.

## Testing

```
cargo test -p dp-game
```

Tests cover name matching, Proton paths, rejected launchers, PID and start-time queries, and cache expiry. Linux tests also scan temporary child processes and check that secondary threads are excluded; they do not launch the game.
