# dp-connection

Streams UDP packet events that the connection monitor uses to see traffic to the game relays. It is ring 1 because it reads kernel network events on Windows and raw packets on Linux and macOS.

## Public API

- `start(config: Config) -> Sender<()>` starts the platform source. Send on the returned sender, or drop it, to stop.
- `Config` holds `wanted`, `sink`, `remotes`, `on_status` and `prompt`.
- `Packet` holds `pid`, `remote`, `inbound` and `ticks_100ns`.
- `Status` is `Failed(String)` or `NeedsPermission`.
- `Wanted`, `Sink`, `Remotes` and `OnStatus` are the callback types in `Config`.
- `HELPER_ARG` is `"--capture-helper"`.
- `run_helper(socket_path) -> i32` is the root side on Linux and macOS. It returns the process exit code.

## Dependencies

- `log`
- `ferrisetw` on Windows
- `libc` on unix
- No workspace crates.

## Platform behaviour

- Windows reads kernel ETW network events in a session named `DeadlockPlusNetwork`. Events carry the process id, so `wanted(pid)` filters before parsing. It needs administrator rights.
- Linux and macOS have no per-process feed. `start` launches the app binary with `--capture-helper <socket>` as root, through `pkexec` on Linux and `osascript` on macOS. The helper captures IPv4 UDP packets with an `AF_PACKET` socket on Linux and a BPF device on macOS. It sends the app only those packets to or from the addresses `remotes` returns. `Packet::pid` is `None` there.
- Any other platform reports `Status::Failed` and delivers nothing.

## Gotchas

- With `prompt` false, the unix source reports `Status::NeedsPermission` and starts nothing. Set `prompt` only for an action the user asked for, because it opens a password dialog.
- The app must call `run_helper` early in `main` when it sees `HELPER_ARG`, and exit with its return value. The caller owns that check.
- The helper connects back over a Unix socket in a `0700` folder under the temp directory. It exits when the app closes the socket. The app waits up to 180 seconds for the helper to connect, so the user has time to type a password.
- Inside an AppImage the helper is started from the `APPIMAGE` file, because root cannot see the image's own mount.
- `ticks_100ns` has no fixed epoch. Use only differences between packets.
- The Windows source clears a session left over from a crashed run with `logman stop DeadlockPlusNetwork -ets` before it starts.
- On Linux the capture socket uses `ETH_P_ALL`. A socket bound to `ETH_P_IP` never sees outgoing packets.

## Testing

```
cargo test -p dp-connection
```

The tests cover the wire format, the packet parser and the address logic. They open no capture device and ask for no rights. The `wire` module compiles on every host. The helper, session and capture code compile on unix only, so run the tests on Linux or macOS to cover them.
