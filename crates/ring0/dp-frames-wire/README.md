# dp-frames-wire

The file format and shared plumbing between the Vulkan frame layer and the app. It is ring 0 because it uses only `std`, so the layer can stay tiny and both sides test it on any host.

## Public API

- `encode_header(pid)` and `encode_record(record, out)` write the format. `Reader::feed(bytes, out)` parses it incrementally, for a file that is still being appended to.
- `Record` is one present call: a monotonic timestamp in nanoseconds and a hash of the swapchain handle.
- `Ring::new(capacity)` is a bounded queue with lock-free `push` and a single-consumer `pop`. A full ring drops the new entry and counts it.
- `data_home`, `frames_dir` and `implicit_layer_dir` resolve the XDG folders the layer writes to and the loader reads from.
- `FRAME_FILE_EXTENSION` is `dpf`.

## Dependencies

None.

## Format

- A 16-byte header: `DPFT`, version `1` as u32, the process id as u32, 4 reserved bytes.
- Then 16-byte records: timestamp u64, swapchain u32, 4 reserved bytes. All little endian.
- The layer writes one file per process, `<pid>.dpf`, in `<data home>/deadlock-plus/frames`.

## Gotchas

- `Reader` returns an error on a wrong magic or version, then ignores all later input.
- A timestamp of zero marks an empty ring slot, so the layer never records zero.
- Records from several threads can be slightly out of order. The app sorts before it computes statistics.

## Testing

```
cargo test -p dp-frames-wire
```
