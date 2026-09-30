# dp-elevation

Reports whether the process has administrator rights. It is ring 1 because it reads the Windows process token.

## Public API

- `is_elevated() -> bool` returns true if the process token is elevated.

## Dependencies

- `windows` on Windows only
- No workspace crates.

## Platform behaviour

- Windows opens the process token and reads `TokenElevation`. Any failure returns `false`.
- Every other platform returns `false` without checking. Only Windows needs elevation up front. The Linux and macOS backends ask for a password when they change firewall rules or start packet capture.

## Testing

```
cargo test -p dp-elevation
```

The crate has no tests. The command only checks that it builds.
