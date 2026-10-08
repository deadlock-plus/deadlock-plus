# dp-icmp

Sends one ICMP echo to an IPv4 address and returns the round-trip time. It is ring 1 because each backend calls an OS facility.

## Public API

- `ping(ip: Ipv4Addr, timeout_ms: u32) -> Option<f32>` returns the round-trip time in milliseconds, or `None` on timeout or any failure.

## Dependencies

- `windows` on Windows only
- No workspace crates.

## Platform behaviour

- Windows calls `IcmpSendEcho` from the IP Helper API. It starts no child process and opens no console window. The result is whole milliseconds.
- Other hosts run the system `ping -c 1 -W <n> <ip>` and parse `time=` from the output. The binary must be on `PATH`.
- The `-W` value is whole seconds. The timeout rounds up to a whole second with a minimum of 1, so `timeout_ms` under 1000 still waits up to one second.

## Gotchas

- The unix module is compiled on every host so its output parser tests run on Windows too. It is exported only on non-Windows targets.
- The Windows reply buffer is a `[u64; 32]` on purpose. `ICMP_ECHO_REPLY` needs 8-byte alignment.

## Testing

```
cargo test -p dp-icmp
```

The tests parse sample `ping` output. They send no packets.
