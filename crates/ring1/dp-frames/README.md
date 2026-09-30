# dp-frames

Frame time capture for the game. It records the DXGI present events of the game process through an ETW session, then turns the timestamps into frame time statistics. It is ring 1 because it talks to a Windows tracing API and to process and window state. It knows nothing about the app.

## Public API

- `FrameCapture` is the handle. Build one with `FrameCapture::default()`.
- `FrameCapture::start(is_game)` starts the trace and two polling threads. `is_game` is `fn(&OsStr) -> bool` and receives a process name. The caller supplies it, so this crate does not depend on `dp-game`. A second `start` while running does nothing.
- `FrameCapture::status()` returns a `CaptureStatus` for the running session, or the default when idle.
- `FrameCapture::stop()` ends the session and returns `FrameStats`.
- `FrameStats::from_segments(segments, ticks_per_second)` computes the statistics from runs of ascending timestamps.
- `split_focused(timestamps, unfocused)` drops frames inside unfocused intervals and splits the rest into segments.
- `recent_frametimes_ms(timestamps, count, ticks_per_second)` returns the newest `count` frame times, oldest first.
- `CaptureState` is `Idle`, `WaitingForGame`, `Capturing` or `Failed`.

TypeScript types exported to `apps/desktop/src/lib/generated/types`: `FrameStats`, `FrameSpike`, `CaptureStatus` and `CaptureState`. The Windows build produces them.

## Dependencies

- `dp-sync`
- `log`, `serde`, `ts-rs`
- Windows only: `ferrisetw`, `sysinfo`, `windows`

## Platform behaviour

- On Windows, `capture.rs` opens an ETW user trace named `DeadlockPlusFrames` on the DXGI provider and counts event id `0x2a` (present start) from the game process. Starting a trace needs administrator rights. Without them the status becomes `Failed` with an error.
- On other platforms, `capture_stub.rs` provides the same types and methods. `start` does nothing, `status` returns the default and `stop` returns empty `FrameStats`. The statistics functions work everywhere.

## Gotchas

- Start clears a stale session first. ETW sessions outlive a crashed process, so the trace thread runs `logman stop DeadlockPlusFrames -ets` before it starts.
- A thread finds the game process id once a second with `is_game`. Until it finds one, the state is `WaitingForGame`. Present events from other processes are only counted in `other_process_events`. A nonzero count with zero `frames` means the game does not present through DXGI.
- A thread polls the foreground window every 50 ms. Frames while the game is in the background are dropped, and so are frames within 250 ms of each side of that interval. The background time is reported as `background_ms`, and no gap between segments counts as a frame.
- The capture keeps at most 1,000,000 frames. After that it sets `truncated` and discards new events.
- `recent_frametimes_ms` in the status covers the newest 300 frames.
- Timestamps are 100 ns FILETIME ticks, so `ticks_per_second` is 10,000,000. A rate of 0 returns empty statistics instead of dividing by zero.
- Percentiles use the nearest-rank method. `low_1pct_fps` and `low_01pct_fps` are 1000 divided by the p99 and p99.9 frame time.
- A frame is a spike when its frame time is above the larger of 2.5 times the median and 33 ms.
- Segments with fewer than two timestamps produce no frames.

## Testing

```
cargo test -p dp-frames
```

The tests cover the statistics, spike detection and focus splitting on synthetic timestamps. They open no trace and need no administrator rights.
